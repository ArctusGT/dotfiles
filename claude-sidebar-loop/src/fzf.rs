//! fzf's `--listen` HTTP API over its unix socket.

use std::io::{self, Read, Write};
use std::os::unix::net::UnixStream;
use std::path::Path;

/// Runs fzf `action` through the socket at `socket`: one `POST /` with the
/// action as its body. fzf answers and closes the connection, so the answer
/// is read to its end. Succeeds on a 200, as `curl -f` would.
pub fn post(socket: &Path, action: &str) -> io::Result<()> {
    let mut stream = UnixStream::connect(socket)?;
    let request = format!(
        "POST / HTTP/1.1\r\nHost: localhost\r\nContent-Length: {}\r\n\r\n{action}",
        action.len()
    );
    stream.write_all(request.as_bytes())?;
    let mut answer = String::new();
    stream.read_to_string(&mut answer)?;
    let status = answer.lines().next().unwrap_or_default();
    if status.split(' ').nth(1) == Some("200") {
        Ok(())
    } else {
        Err(io::Error::other(format!("fzf answered {status:?}")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::net::UnixListener;
    use std::thread;

    /// a one-request fzf answering `answer`, returning the request it got
    fn fake_fzf(answer: &'static str) -> (crate::tempdir::Dir, thread::JoinHandle<String>) {
        let dir = crate::tempdir::Dir::new();
        let listener = UnixListener::bind(dir.path().join("fzf.sock")).unwrap();
        let served = thread::spawn(move || {
            let (mut conn, _) = listener.accept().unwrap();
            let mut request = vec![0; 4096];
            let n = conn.read(&mut request).unwrap();
            conn.write_all(answer.as_bytes()).unwrap();
            String::from_utf8(request[..n].to_vec()).unwrap()
        });
        (dir, served)
    }

    #[test]
    fn posts_the_action_as_the_body() {
        let (dir, served) = fake_fzf("HTTP/1.1 200 OK\r\n\r\n");
        post(&dir.path().join("fzf.sock"), "pos(3)").unwrap();
        assert_eq!(
            served.join().unwrap(),
            "POST / HTTP/1.1\r\nHost: localhost\r\nContent-Length: 6\r\n\r\npos(3)"
        );
    }

    #[test]
    fn fails_on_an_error_answer() {
        let (dir, served) =
            fake_fzf("HTTP/1.1 400 Bad Request\r\nContent-Length: 15\r\n\r\nunknown action\n");
        assert!(post(&dir.path().join("fzf.sock"), "nope(").is_err());
        served.join().unwrap();
    }

    #[test]
    fn fails_with_no_fzf() {
        let dir = crate::tempdir::Dir::new();
        assert!(post(&dir.path().join("fzf.sock"), "pos(1)").is_err());
    }
}
