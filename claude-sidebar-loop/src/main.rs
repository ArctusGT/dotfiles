//! `claude-sidebar` starts this as one of its loops; see `USAGE`.

mod fzf;
mod serve;

use serve::{Serve, Sidebar};
use std::fs::{self, OpenOptions};
use std::io::{self, BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError};
use std::thread;
use std::time::{Duration, Instant};

const USAGE: &str = "\
claude-sidebar-loop CACHE: show the frames claude-sidebar draws into its cache
directory CACHE in the sidebar's fzf, through fzf's socket CACHE/fzf.sock:
after each gather's line on the fifo CACHE/gathered, and 8 frames a second
while a Clawd moves. claude-sidebar starts it; it runs until killed.";

/// 8 frames a second
const TICK: Duration = Duration::from_millis(125);

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let [cache] = args.as_slice() else {
        eprintln!("{USAGE}");
        std::process::exit(2);
    };
    if cache == "--help" {
        println!("{USAGE}");
        return;
    }
    let cache = Cache { dir: PathBuf::from(cache) };
    let gathers = match lines(&cache.dir.join("gathered")) {
        Ok(gathers) => gathers,
        Err(err) => {
            eprintln!("claude-sidebar-loop: {}/gathered: {err}", cache.dir.display());
            std::process::exit(1);
        }
    };
    serve(Serve::new(cache), &gathers);
}

/// waits for each gather's line, or, while a Clawd moves, for that or the
/// next tick, and hands it to `serve`. Ticks keep to a schedule, so a slow
/// reload does not slow the frame rate; behind, the missed ones are dropped
fn serve(mut serve: Serve<Cache>, gathers: &Receiver<String>) {
    let mut next_tick = Instant::now();
    loop {
        if !serve.animating() {
            let Ok(line) = gathers.recv() else { return };
            next_tick = Instant::now() + TICK;
            serve.gathered(&line);
            continue;
        }
        let delay = next_tick.saturating_duration_since(Instant::now());
        if !delay.is_zero() {
            match gathers.recv_timeout(delay) {
                Ok(line) => {
                    serve.gathered(&line);
                    continue;
                }
                Err(RecvTimeoutError::Timeout) => {}
                Err(RecvTimeoutError::Disconnected) => return,
            }
        }
        let now = Instant::now();
        next_tick += TICK;
        if next_tick <= now {
            next_tick = now + TICK;
        }
        serve.tick();
    }
}

/// the lines written to the fifo `fifo`, as they come. Opened read and
/// write, so the fifo always has a writer and a read waits for a line
/// rather than seeing end of file
fn lines(fifo: &Path) -> io::Result<Receiver<String>> {
    let file = OpenOptions::new().read(true).write(true).open(fifo)?;
    let (send, receive) = mpsc::channel();
    thread::spawn(move || {
        for line in BufReader::new(file).lines() {
            let Ok(line) = line else { return };
            if send.send(line).is_err() {
                return;
            }
        }
    });
    Ok(receive)
}

/// the sidebar's cache directory, where `claude-sidebar` leaves the rows,
/// `repeats`, `jumped`, the `moving.*` marks and fzf's socket
struct Cache {
    dir: PathBuf,
}

impl Cache {
    fn rows_path(&self, frame: usize) -> PathBuf {
        self.dir.join(format!("rows.{frame}"))
    }

    /// the file `name`, or nothing, with the error logged
    fn read(&self, name: &Path) -> Vec<u8> {
        fs::read(name).unwrap_or_else(|err| {
            eprintln!("claude-sidebar-loop: {}: {err}", name.display());
            Vec::new()
        })
    }

    /// whether fzf took `action`; a sidebar starting before its fzf listens
    /// fails quietly, as the next gather tries again
    fn post(&self, action: &str) -> bool {
        fzf::post(&self.dir.join("fzf.sock"), action).is_ok()
    }
}

impl Sidebar for Cache {
    fn rows(&self, frame: usize) -> Vec<u8> {
        self.read(&self.rows_path(frame))
    }

    /// `repeats` holds a 1 or 0 for each frame
    fn repeats(&self) -> Vec<bool> {
        let repeats = self.read(&self.dir.join("repeats"));
        String::from_utf8_lossy(&repeats).split_whitespace().map(|r| r == "1").collect()
    }

    fn jumped(&self) -> String {
        let jumped = self.read(&self.dir.join("jumped"));
        String::from_utf8_lossy(&jumped).lines().next().unwrap_or_default().trim().to_string()
    }

    /// a jump marks itself `moving.<pid>` until its client has moved
    fn moving(&self) -> bool {
        fs::read_dir(&self.dir).is_ok_and(|entries| {
            entries.flatten().any(|e| e.file_name().as_encoded_bytes().starts_with(b"moving."))
        })
    }

    fn reload(&mut self, frame: usize) -> bool {
        self.post(&format!("reload-sync(cat '{}')", self.rows_path(frame).display()))
    }

    fn move_cursor(&mut self, position: usize) -> bool {
        self.post(&format!("pos({position})"))
    }
}

/// A temporary directory for tests, removed when dropped. Short, under
/// /tmp, as a unix socket's path must fit in 104 bytes on macOS.
#[cfg(test)]
mod tempdir {
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicU32, Ordering};

    pub struct Dir(PathBuf);

    impl Dir {
        pub fn new() -> Dir {
            static NEXT: AtomicU32 = AtomicU32::new(0);
            let n = NEXT.fetch_add(1, Ordering::Relaxed);
            let path = PathBuf::from(format!("/tmp/csl-{}-{n}", std::process::id()));
            std::fs::create_dir(&path).unwrap();
            Dir(path)
        }

        pub fn path(&self) -> &Path {
            &self.0
        }
    }

    impl Drop for Dir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_the_cache_as_claude_sidebar_writes_it() {
        let dir = tempdir::Dir::new();
        let cache = Cache { dir: dir.path().into() };
        fs::write(dir.path().join("repeats"), "1 0 1\n").unwrap();
        fs::write(dir.path().join("jumped"), "1760000000.123456-42\n").unwrap();
        assert_eq!(cache.repeats(), [true, false, true]);
        assert_eq!(cache.jumped(), "1760000000.123456-42");
        assert!(!cache.moving());
        fs::write(dir.path().join("moving.42"), "").unwrap();
        assert!(cache.moving());
    }
}
