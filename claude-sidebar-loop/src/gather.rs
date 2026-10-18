//! One gather: every host's sessions, tagged with where they came from,
//! this host's read in-process (`sessions`), each remote host's from its
//! own `claude-sessions` over ssh, all hosts at once, so an unreachable one
//! costs one ssh ConnectTimeout, not more.
//!
//! The tagged records are what `frames::render` draws: for each host, a
//! header, `{header: true, label, down, current}`, `down` when the host did
//! not answer and `current` the host's pane on show in the frame's right
//! pane, if any; then its sessions with a pane first, desktop app sessions
//! left out unless one has a pane, each with `host` (its ssh host, empty
//! for this one) and `relay` (the Mac's tmux session showing that host).

use crate::json::{self, Value};
use crate::sessions::{self, Sources};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::thread;

/// ServerAlive drops a master connection to a host gone quiet within 4s,
/// where ConnectTimeout does not reach a call made over an existing master;
/// as `claude-sidebar`'s `ssh_opts`, so both share one master per host
const SSH_OPTS: [&str; 10] = [
    "-o",
    "BatchMode=yes",
    "-o",
    "ConnectTimeout=3",
    "-o",
    "ServerAliveInterval=2",
    "-o",
    "ServerAliveCountMax=2",
    "-o",
    "ControlPath=/tmp/claude-sidebar-%C",
];

/// run on each remote host: its sessions, then \036 and the pane its one
/// client from the Mac (which runs inside the Mac's tmux, a tmux* term) is on
const REMOTE_COMMAND: &str = "claude-sessions; printf '\\036'; \
    tmux list-clients -f '#{m:tmux*,#{client_termname}}' -F '#{pane_id}' | head -1";

/// A host a sesh entry reaches through `ssh-tmux`.
struct Remote {
    /// the sesh session's name: the host's heading and its relay session
    name: String,
    ssh_host: String,
}

/// every host's tagged records, this host's first
pub fn gather() -> Vec<Value> {
    let (shown, relay_host) = on_show();
    let remotes = remotes();
    thread::scope(|scope| {
        let polls: Vec<_> = remotes
            .iter()
            .map(|remote| scope.spawn(|| poll(remote, &relay_host)))
            .collect();
        let local_shown = if relay_host.is_empty() {
            shown
        } else {
            String::new()
        };
        let mut tagged = vec![header("local", false, local_shown)];
        tagged.extend(tag(sessions::read(&Sources::from_env()), "", ""));
        for poll in polls {
            tagged.extend(poll.join().unwrap_or_default());
        }
        tagged
    })
}

/// what the frame's right pane shows: its pane on the usual tmux, and the
/// ssh host whose relay window that is, if one is; both empty outside the
/// frame
fn on_show() -> (String, String) {
    let Some(tty) = frame_main_tty() else {
        return Default::default();
    };
    let shown = output(
        Command::new("tmux")
            .args(["display", "-p", "-c", &tty, "#{pane_id} #{@ssh_tmux}"])
            .env_remove("TMUX")
            .env_remove("TMUX_PANE"),
    );
    let line = shown.lines().next().unwrap_or_default().trim();
    match line.split_once([' ', '\t']) {
        Some((pane, host)) => (pane.to_string(), host.trim().to_string()),
        None => (line.to_string(), String::new()),
    }
}

/// the tty of the frame's right pane, as `claude-sidebar`'s `frame_main`
/// finds it, in the tmux this runs in
fn frame_main_tty() -> Option<String> {
    let panes = output(Command::new("tmux").args([
        "list-panes",
        "-s",
        "-F",
        "#{@frame_main} #{pane_id} #{pane_tty}",
    ]));
    panes.lines().find_map(|line| {
        let mut words = line.split_whitespace();
        (words.next() == Some("1"))
            .then(|| words.nth(1).map(str::to_string))
            .flatten()
    })
}

/// `command`'s standard output, empty if it could not run
fn output(command: &mut Command) -> String {
    command
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .output()
        .map(|out| String::from_utf8_lossy(&out.stdout).into_owned())
        .unwrap_or_default()
}

/// the hosts the sesh config's `ssh-tmux` entries reach, in its order
fn remotes() -> Vec<Remote> {
    let config = std::env::var_os("SESH_CONFIG")
        .filter(|v| !v.is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(std::env::var_os("HOME").unwrap_or_default())
                .join(".config/sesh/sesh.toml")
        });
    std::fs::read_to_string(config)
        .map(|text| parse_remotes(&text))
        .unwrap_or_default()
}

/// each `[[session]]` whose `startup_command` runs `ssh-tmux HOST`: its name
/// and HOST, read line by line as the config writes them
fn parse_remotes(config: &str) -> Vec<Remote> {
    let mut remotes = Vec::new();
    let mut name = String::new();
    for line in config.lines() {
        if line.starts_with("[[session]]") {
            name.clear();
        }
        if let Some(value) = assignment(line, "name") {
            let value = value.strip_prefix('"').unwrap_or(value);
            name = value.split('"').next().unwrap_or_default().to_string();
        }
        if let Some(value) = assignment(line, "startup_command")
            && let Some(at) = value.rfind("ssh-tmux ")
        {
            let host = value[at + "ssh-tmux ".len()..].trim_start_matches(' ');
            remotes.push(Remote {
                name: name.clone(),
                ssh_host: host
                    .split([' ', '"'])
                    .next()
                    .unwrap_or_default()
                    .to_string(),
            });
        }
    }
    remotes
}

/// the value of a `key = value` line, from just past the `=` and its spaces
fn assignment<'a>(line: &'a str, key: &str) -> Option<&'a str> {
    let rest = line.strip_prefix(key)?.trim_start_matches(' ');
    Some(rest.strip_prefix('=')?.trim_start_matches(' '))
}

/// `remote`'s tagged records: its sessions, and its pane on show where the
/// frame's right pane shows its relay window (`relay_host`); its header
/// marked down if ssh failed. One master connection per host, so a poll is
/// not a login
fn poll(remote: &Remote, relay_host: &str) -> Vec<Value> {
    let answer = Command::new("ssh")
        .args(SSH_OPTS)
        .args(["-o", "ControlMaster=auto", "-o", "ControlPersist=60"])
        .args([&remote.ssh_host, REMOTE_COMMAND])
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .output();
    let out = match answer {
        Ok(out) if out.status.success() => String::from_utf8_lossy(&out.stdout).into_owned(),
        _ => return vec![header(&remote.name, true, String::new())],
    };
    let (sessions, client_pane) = out.split_once('\u{1e}').unwrap_or((&out, &out));
    let shown = if relay_host == remote.ssh_host {
        client_pane.replace('\n', "")
    } else {
        String::new()
    };
    let mut tagged = vec![header(&remote.name, false, shown)];
    // an answer that is not all JSON shows as a host with no sessions
    if let Ok(sessions) = json::parse_stream(sessions) {
        tagged.extend(tag(sessions, &remote.ssh_host, &remote.name));
    }
    tagged
}

fn header(label: &str, down: bool, current: String) -> Value {
    Value::Object(vec![
        ("header".into(), Value::Bool(true)),
        ("label".into(), Value::String(label.into())),
        ("down".into(), Value::Bool(down)),
        ("current".into(), Value::String(current)),
    ])
}

/// a host's `sessions` as the sidebar lists them: those with a pane first,
/// desktop app sessions left out unless one has a pane, each tagged with
/// its ssh host and relay session
fn tag(sessions: Vec<Value>, host: &str, relay: &str) -> Vec<Value> {
    let mut tagged: Vec<Value> = sessions
        .into_iter()
        .filter(|s| {
            s.get("entrypoint").as_str() != Some("claude-desktop")
                || s.get("tmux_pane") != &Value::Null
        })
        .collect();
    tagged.sort_by_key(|s| s.get("tmux_pane") == &Value::Null);
    for session in &mut tagged {
        session.set("host", Value::String(host.into()));
        session.set("relay", Value::String(relay.into()));
    }
    tagged
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_the_ssh_tmux_hosts_from_the_sesh_config() {
        let remotes = parse_remotes(
            "[[session]]\nname = \"mgmt-1\"\npath = \"~\"\n\
             startup_command = \"ssh-tmux sysadmin@mgmt-1\"\n\n\
             [[session]]\nname = \"dotfiles\"\nstartup_command = \"nvim\"\n\
             [[session]]\nname = \"lab\"\nstartup_command = \"ssh-tmux   lab-2 -t x\"\n",
        );
        let found: Vec<(&str, &str)> = remotes
            .iter()
            .map(|r| (r.name.as_str(), r.ssh_host.as_str()))
            .collect();
        assert_eq!(found, [("mgmt-1", "sysadmin@mgmt-1"), ("lab", "lab-2")]);
    }

    #[test]
    fn tags_sessions_with_a_pane_first_and_no_paneless_desktop_ones() {
        let sessions = json::parse_stream(
            "{\"session_id\":\"a\",\"tmux_pane\":null,\"entrypoint\":\"cli\"}\
             {\"session_id\":\"b\",\"tmux_pane\":null,\"entrypoint\":\"claude-desktop\"}\
             {\"session_id\":\"c\",\"tmux_pane\":\"%1\",\"entrypoint\":\"claude-desktop\"}\
             {\"session_id\":\"d\",\"tmux_pane\":\"%2\",\"entrypoint\":\"cli\"}",
        )
        .unwrap();
        let tagged: Vec<String> = tag(sessions, "h", "r")
            .iter()
            .map(Value::to_string)
            .collect();
        assert_eq!(
            tagged,
            [
                "{\"session_id\":\"c\",\"tmux_pane\":\"%1\",\"entrypoint\":\"claude-desktop\",\"host\":\"h\",\"relay\":\"r\"}",
                "{\"session_id\":\"d\",\"tmux_pane\":\"%2\",\"entrypoint\":\"cli\",\"host\":\"h\",\"relay\":\"r\"}",
                "{\"session_id\":\"a\",\"tmux_pane\":null,\"entrypoint\":\"cli\",\"host\":\"h\",\"relay\":\"r\"}",
            ]
        );
    }
}
