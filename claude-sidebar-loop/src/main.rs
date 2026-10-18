//! `claude-sidebar` starts this as one of its loops; see `USAGE`.

mod frames;
mod fzf;
mod gather;
mod json;
mod serve;
mod sessions;

use frames::{FRAME_COUNT, View};
use json::Value;
use serve::{Gather, Serve, Sidebar};
use std::fs::{self, File, OpenOptions};
use std::io::{self, BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::thread;
use std::time::{Duration, Instant, SystemTime};

const USAGE: &str = "\
claude-sidebar-loop CACHE: gather the sidebar's sessions every second, or
at once on a line through the fifo CACHE/kick, render each gather's frames
into CACHE/rows.0 on, a file a frame, and show them in the sidebar's fzf,
through fzf's socket CACHE/fzf.sock, 8 frames a second while a Clawd moves.
claude-sidebar starts it; it runs until killed.
claude-sidebar-loop rows: gather once and print the list, as fzf reads it.
claude-sidebar-loop sessions: print this host's sessions, as claude-sessions
does.
A gather reads this host's sessions from the state files in
${XDG_STATE_HOME:-~/.local/state}/claude-sessions, Claude Code's records in
~/.claude/sessions and the default tmux server's panes, removing a dead
session's files, as claude-sessions does, and runs claude-sessions over ssh
on each host the sesh config SESH_CONFIG (default ~/.config/sesh/sesh.toml)
reaches through ssh-tmux. The list is drawn the width of the terminal, a
context line red at CLAUDE_CTX_WARN percent (default 50).";

/// 8 frames a second
const TICK: Duration = Duration::from_millis(125);

/// how long a gather waits for a kick before the next
const GATHER_EVERY: Duration = Duration::from_secs(1);

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let [arg] = args.as_slice() else {
        eprintln!("{USAGE}");
        std::process::exit(2);
    };
    match arg.as_str() {
        "--help" => println!("{USAGE}"),
        "rows" => {
            if let Err(err) = print_rows() {
                eprintln!("claude-sidebar-loop: {err}");
                std::process::exit(1);
            }
        }
        "sessions" => {
            for session in sessions::read(&sessions::Sources::from_env()) {
                println!("{session}");
            }
        }
        cache => run(Cache {
            dir: PathBuf::from(cache),
        }),
    }
}

/// renders frame 0 of a gather to stdout
fn print_rows() -> Result<(), String> {
    let rows = render(&gather::gather(), 1)?;
    io::stdout()
        .write_all(&rows[0])
        .map_err(|err| err.to_string())
}

fn run(cache: Cache) {
    let kicks = match kicks(&cache.dir) {
        Ok(kicks) => kicks,
        Err(err) => {
            eprintln!("claude-sidebar-loop: {}/kick: {err}", cache.dir.display());
            std::process::exit(1);
        }
    };
    let (send, gathers) = mpsc::channel();
    let gathering_cache = cache.clone();
    thread::spawn(move || gathering(&gathering_cache, &kicks, &send));
    let dir = cache.dir.clone();
    serve(Serve::new(cache), &gathers, &dir);
}

/// the first `count` frames of the tagged records `tagged`, for this
/// terminal, now
fn render(tagged: &[Value], count: usize) -> Result<Vec<Vec<u8>>, String> {
    let view = View {
        width: line_width(),
        warn: warn_level()?,
        now: SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map_or(0, |d| d.as_secs() as i64),
    };
    Ok(frames::render(tagged, &view, count)
        .into_iter()
        .map(String::into_bytes)
        .collect())
}

/// CLAUDE_CTX_WARN, 50 when unset or empty, as bash's `${CLAUDE_CTX_WARN:-50}`
fn warn_level() -> Result<f64, String> {
    match std::env::var("CLAUDE_CTX_WARN") {
        Ok(w) if !w.is_empty() => w
            .parse()
            .map_err(|_| "CLAUDE_CTX_WARN is not a number".into()),
        _ => Ok(50.0),
    }
}

/// the columns an fzf item's line has in this terminal, past the 2 of fzf's
/// pointer and marker; read on each render, so the lines follow a resize.
/// 80 columns with no terminal, as `tput cols` takes
fn line_width() -> i64 {
    terminal_columns().unwrap_or(80) as i64 - 2
}

/// the columns of this process's controlling terminal, as its pane has them
fn terminal_columns() -> Option<u16> {
    #[repr(C)]
    #[derive(Default)]
    struct Winsize {
        rows: u16,
        cols: u16,
        x_pixels: u16,
        y_pixels: u16,
    }
    unsafe extern "C" {
        fn ioctl(fd: std::ffi::c_int, request: std::ffi::c_ulong, ...) -> std::ffi::c_int;
    }
    #[cfg(target_os = "macos")]
    const TIOCGWINSZ: std::ffi::c_ulong = 0x40087468;
    #[cfg(target_os = "linux")]
    const TIOCGWINSZ: std::ffi::c_ulong = 0x5413;
    use std::os::fd::AsRawFd;
    let tty = File::open("/dev/tty").ok()?;
    let mut size = Winsize::default();
    // SAFETY: TIOCGWINSZ writes one winsize, which `size` is, to a live fd
    let ok = unsafe { ioctl(tty.as_raw_fd(), TIOCGWINSZ, &mut size as *mut Winsize) } == 0;
    (ok && size.cols > 0).then_some(size.cols)
}

/// hands `serve` each gather, its rows written to the cache `dir` in the
/// same step, and while a Clawd moves each tick between
fn serve(mut serve: Serve<Cache>, gathers: &Receiver<Gather>, dir: &Path) {
    let mut next_tick = Instant::now();
    loop {
        match next_event(serve.animating(), &mut next_tick, gathers) {
            Some(Event::Gather(gather)) => {
                if let Err(err) = write_rows(dir, &gather.frames) {
                    eprintln!("claude-sidebar-loop: {err}");
                }
                serve.gathered(gather)
            }
            Some(Event::Tick) => serve.tick(),
            None => return,
        }
    }
}

enum Event {
    Gather(Gather),
    Tick,
}

/// waits for the next gather, or, while `animating`, for that or the tick
/// at `next_tick`, then moves `next_tick` on: a tick after a gather while
/// still, a tick after a tick. Ticks keep to a schedule, so a slow reload
/// does not slow the frame rate; behind, the missed ones are dropped, and
/// the tick comes before a gather waiting. None once the gathering has
/// stopped
fn next_event(
    animating: bool,
    next_tick: &mut Instant,
    gathers: &Receiver<Gather>,
) -> Option<Event> {
    if !animating {
        let gather = gathers.recv().ok()?;
        *next_tick = Instant::now() + TICK;
        return Some(Event::Gather(gather));
    }
    let delay = next_tick.saturating_duration_since(Instant::now());
    if !delay.is_zero() {
        match gathers.recv_timeout(delay) {
            Ok(gather) => return Some(Event::Gather(gather)),
            Err(RecvTimeoutError::Timeout) => {}
            Err(RecvTimeoutError::Disconnected) => return None,
        }
    }
    let now = Instant::now();
    *next_tick += TICK;
    if *next_tick <= now {
        *next_tick = now + TICK;
    }
    Some(Event::Tick)
}

/// the kicks `claude-sidebar kick` sends, a line each, through the fifo
/// `kick` in the cache `dir`, as they come. Opened read and write, so the
/// fifo always has a writer and a read waits for a line rather than seeing
/// end of file
fn kicks(dir: &Path) -> io::Result<Receiver<()>> {
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .open(dir.join("kick"))?;
    let (send, receive) = mpsc::channel();
    thread::spawn(move || {
        for line in BufReader::new(file).lines() {
            if line.is_err() || send.send(()).is_err() {
                return;
            }
        }
    });
    Ok(receive)
}

/// gathers every second, or as soon as a kick comes, and hands each gather
/// to `send`, rendered, with the value `jumped` had when it began, and
/// whether a kick came during it, so what it saw on show in the frame may
/// already be gone; then gathers again at once. Returns once nothing takes
/// the gathers
fn gathering(cache: &Cache, kicks: &Receiver<()>, send: &Sender<Gather>) {
    loop {
        drained(kicks); // the kicks this gather answers
        let since = cache.jumped();
        let frames = render(&gather::gather(), FRAME_COUNT);
        let moved = drained(kicks);
        match frames {
            Ok(frames) => {
                if send
                    .send(Gather {
                        since,
                        moved,
                        frames,
                    })
                    .is_err()
                {
                    return;
                }
            }
            Err(err) => eprintln!("claude-sidebar-loop: {err}"),
        }
        if !moved && kicks.recv_timeout(GATHER_EVERY) == Err(RecvTimeoutError::Disconnected) {
            return;
        }
    }
}

/// empties `kicks` of the kicks waiting; true if there were any
fn drained(kicks: &Receiver<()>) -> bool {
    kicks.try_iter().count() > 0
}

/// each of `frames` to its `rows.N` in `dir`, replaced whole, as
/// `reload-sync` and `claude-sidebar`'s `step` and `position` read them
fn write_rows(dir: &Path, frames: &[Vec<u8>]) -> Result<(), String> {
    for (k, rows) in frames.iter().enumerate() {
        let path = rows_path(dir, k);
        let new = dir.join(format!("rows.{k}.new"));
        fs::write(&new, rows)
            .and_then(|_| fs::rename(&new, &path))
            .map_err(|err| format!("{}: {err}", path.display()))?;
    }
    Ok(())
}

fn rows_path(dir: &Path, frame: usize) -> PathBuf {
    dir.join(format!("rows.{frame}"))
}

/// the file at `path`, or nothing, with the error logged
fn read_logged(path: &Path) -> Vec<u8> {
    fs::read(path).unwrap_or_else(|err| {
        eprintln!("claude-sidebar-loop: {}: {err}", path.display());
        Vec::new()
    })
}

/// the sidebar's cache directory, where `claude-sidebar` leaves the kick
/// fifo, `jumped`, the `moving.*` marks and fzf's socket, and this leaves
/// the rows
#[derive(Clone)]
struct Cache {
    dir: PathBuf,
}

impl Cache {
    /// whether fzf took `action`; a sidebar starting before its fzf listens
    /// fails quietly, as the next gather tries again
    fn post(&self, action: &str) -> bool {
        fzf::post(&self.dir.join("fzf.sock"), action).is_ok()
    }
}

impl Sidebar for Cache {
    fn jumped(&self) -> String {
        let jumped = read_logged(&self.dir.join("jumped"));
        String::from_utf8_lossy(&jumped)
            .lines()
            .next()
            .unwrap_or_default()
            .trim()
            .to_string()
    }

    /// a jump marks itself `moving.<pid>` until its client has moved
    fn moving(&self) -> bool {
        fs::read_dir(&self.dir).is_ok_and(|entries| {
            entries
                .flatten()
                .any(|e| e.file_name().as_encoded_bytes().starts_with(b"moving."))
        })
    }

    fn reload(&mut self, frame: usize) -> bool {
        self.post(&format!(
            "reload-sync(cat '{}')",
            rows_path(&self.dir, frame).display()
        ))
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
        let cache = Cache {
            dir: dir.path().into(),
        };
        fs::write(dir.path().join("jumped"), "1760000000.123456-42\n").unwrap();
        assert_eq!(cache.jumped(), "1760000000.123456-42");
        assert!(!cache.moving());
        fs::write(dir.path().join("moving.42"), "").unwrap();
        assert!(cache.moving());
    }

    #[test]
    fn writes_a_rows_file_a_frame() {
        let dir = tempdir::Dir::new();
        let tagged = json::parse_stream(
            "{\"header\":true,\"label\":\"local\",\"down\":false,\"current\":\"\"}\n",
        )
        .unwrap();
        let frames = render(&tagged, FRAME_COUNT).unwrap();
        write_rows(dir.path(), &frames).unwrap();
        assert_eq!(frames.len(), FRAME_COUNT);
        for (k, rows) in frames.iter().enumerate() {
            assert_eq!(&fs::read(rows_path(dir.path(), k)).unwrap(), rows);
        }
        assert!(String::from_utf8_lossy(&frames[0]).contains("local"));
    }

    #[test]
    fn a_kick_during_a_gather_is_drained_and_reported() {
        let (send, kicks) = mpsc::channel();
        assert!(!drained(&kicks));
        send.send(()).unwrap();
        send.send(()).unwrap();
        assert!(drained(&kicks));
        assert!(!drained(&kicks));
    }
}
