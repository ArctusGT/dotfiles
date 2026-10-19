//! One gather: every host's sessions, this host's read in-process
//! (`sessions`), each remote host's from its own `claude-sessions` over ssh,
//! all hosts at once, so an unreachable one costs one ssh ConnectTimeout,
//! not more.

use crate::json::{self, Value};
use crate::session::Session;
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
/// client from the Mac (which runs inside the Mac's tmux, a tmux* term) is on,
/// found as `claude-sidebar`'s `relay_client` finds it
const REMOTE_COMMAND: &str = "claude-sessions; printf '\\036'; \
    tmux list-clients -f '#{m:tmux*,#{client_termname}}' -F '#{pane_id}' | head -1";

/// A host's part of a gather, as the sidebar lists it.
pub struct Host {
    /// its heading: `local`, or its sesh session's name
    pub label: String,
    /// its ssh host; empty for this one
    pub ssh_host: String,
    /// the Mac's tmux session showing it, its sesh session; empty for this
    /// one
    pub relay: String,
    /// whether it did not answer
    pub down: bool,
    /// its pane the frame's right pane shows, if any
    pub on_show: Option<String>,
    /// its sessions, grouped by repo as `listed` orders them
    pub sessions: Vec<Session>,
}

impl Host {
    /// this host, with `sessions`
    fn local(on_show: &OnShow, sessions: Vec<Value>) -> Host {
        let on_show = match on_show {
            OnShow::Local(pane) => Some(pane.clone()),
            _ => None,
        };
        Host {
            label: "local".into(),
            ssh_host: String::new(),
            relay: String::new(),
            down: false,
            sessions: listed(sessions, on_show.as_deref()),
            on_show,
        }
    }
}

/// A host a sesh entry reaches through `ssh-tmux`.
struct Remote {
    /// the sesh session's name: the host's heading and its relay session
    name: String,
    ssh_host: String,
}

/// What the frame's right pane shows.
#[derive(Default)]
enum OnShow {
    /// nothing: no frame, or tmux did not say
    #[default]
    Nothing,
    /// a pane of the usual tmux on this host
    Local(String),
    /// a host's relay window, by its ssh host
    Relay(String),
}

/// every host, this one first
pub fn gather() -> Vec<Host> {
    let on_show = on_show();
    let remotes = remotes();
    thread::scope(|scope| {
        let polls: Vec<_> = remotes
            .iter()
            .map(|remote| (remote, scope.spawn(|| poll(remote, &on_show))))
            .collect();
        let mut hosts = vec![Host::local(
            &on_show,
            sessions::read(&Sources::from_env()),
        )];
        for (remote, poll) in polls {
            hosts.push(poll.join().unwrap_or_else(|_| remote.host(true)));
        }
        hosts
    })
}

/// what the frame's right pane shows, from its pane on the usual tmux and
/// the ssh host its window relays, if it does
fn on_show() -> OnShow {
    let Some(tty) = frame_main_tty() else {
        return OnShow::Nothing;
    };
    let shown = output(
        Command::new("tmux")
            .args(["display", "-p", "-c", &tty, "#{pane_id} #{@ssh_tmux}"])
            .env_remove("TMUX")
            .env_remove("TMUX_PANE"),
    );
    let line = shown.lines().next().unwrap_or_default().trim();
    let (pane, host) = line.split_once([' ', '\t']).unwrap_or((line, ""));
    match (pane, host.trim()) {
        ("", _) => OnShow::Nothing,
        (pane, "") => OnShow::Local(pane.to_string()),
        (_, host) => OnShow::Relay(host.to_string()),
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
        words.next().filter(|&word| word == "1")?;
        words.nth(1).map(str::to_string)
    })
}

/// `command`'s standard output, empty if it could not run
pub fn output(command: &mut Command) -> String {
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
        if let Some((_, host)) =
            assignment(line, "startup_command").and_then(|value| value.rsplit_once("ssh-tmux "))
        {
            let host = host.trim_start_matches(' ').split([' ', '"']).next();
            remotes.push(Remote {
                name: name.clone(),
                ssh_host: host.unwrap_or_default().to_string(),
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

impl Remote {
    /// the host, with no sessions
    fn host(&self, down: bool) -> Host {
        Host {
            label: self.name.clone(),
            ssh_host: self.ssh_host.clone(),
            relay: self.name.clone(),
            down,
            on_show: None,
            sessions: Vec::new(),
        }
    }
}

/// `remote` as it answers: its sessions, and its pane on show where the
/// frame's right pane shows its relay window; down if ssh failed. One master
/// connection per host, so a poll is not a login
fn poll(remote: &Remote, on_show: &OnShow) -> Host {
    let answer = Command::new("ssh")
        .args(SSH_OPTS)
        .args(["-o", "ControlMaster=auto", "-o", "ControlPersist=60"])
        .args([&remote.ssh_host, REMOTE_COMMAND])
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .output();
    let out = match answer {
        Ok(out) if out.status.success() => String::from_utf8_lossy(&out.stdout).into_owned(),
        _ => return remote.host(true),
    };
    // with no \036, both halves are the whole answer, as bash's
    // ${out%%$'\036'*} and ${out#*$'\036'} take it
    let (sessions, client_pane) = out.split_once('\u{1e}').unwrap_or((&out, &out));
    let mut host = remote.host(false);
    if matches!(on_show, OnShow::Relay(h) if *h == remote.ssh_host) {
        host.on_show = Some(client_pane.replace('\n', "")).filter(|p| !p.is_empty());
    }
    // an answer that is not all JSON shows as a host with no sessions
    if let Ok(sessions) = json::parse_stream(sessions) {
        host.sessions = listed(sessions, host.on_show.as_deref());
    }
    host
}

/// a host's `sessions` (the objects `claude-sessions` prints) as the sidebar
/// lists them, desktop app sessions left out unless one has a pane: grouped
/// by repo, `~` first, then the rest of HOME, then the rest, each group in
/// order of its path, case aside. In a group, the one on show in pane
/// `on_show` first, then the others with a pane, then those with none, each
/// in the order `claude-sessions` printed them. One that does not parse is
/// logged and left out
fn listed(sessions: Vec<Value>, on_show: Option<&str>) -> Vec<Session> {
    let mut listed: Vec<Session> = sessions
        .iter()
        .filter_map(|s| {
            Session::parse(s)
                .map_err(|err| eprintln!("claude-sidebar-loop: a session left out: {err}"))
                .ok()
        })
        .filter(|s| !s.desktop || s.pane.is_some())
        .collect();
    listed.sort_by_cached_key(|s| {
        let repo = s.repo.as_deref();
        let group = match repo {
            Some("~") => 0,
            Some(r) if r.starts_with("~/") => 1,
            Some(_) => 2,
            None => 3,
        };
        let place = match s.pane.as_deref() {
            Some(pane) if Some(pane) == on_show => 0,
            Some(_) => 1,
            None => 2,
        };
        (group, repo.unwrap_or_default().to_lowercase(), place)
    });
    listed
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

    /// a session object of id `id`, pane `pane` (JSON) and entrypoint `entry`
    fn session(id: &str, pane: &str, entry: &str) -> String {
        format!(
            "{{\"session_id\":\"{id}\",\"tmux_pane\":{pane},\"entrypoint\":\"{entry}\",\
             \"state\":\"idle\",\"since\":1,\"subagents\":0}}"
        )
    }

    #[test]
    fn lists_sessions_with_a_pane_first_and_no_paneless_desktop_ones() {
        let sessions = json::parse_stream(
            &[
                session("a", "null", "cli"),
                session("b", "null", "claude-desktop"),
                session("c", "\"%1\"", "claude-desktop"),
                session("d", "\"%2\"", "cli"),
            ]
            .concat(),
        )
        .unwrap();
        let ids: Vec<String> = listed(sessions, None).into_iter().map(|s| s.id).collect();
        assert_eq!(ids, ["c", "d", "a"]);
    }

    #[test]
    fn groups_sessions_by_repo_home_first_with_the_one_on_show_leading() {
        let in_repo = |id: &str, pane: &str, repo: &str| {
            session(id, pane, "cli").replace("}", &format!(",\"repo\":{repo}}}"))
        };
        let sessions = json::parse_stream(
            &[
                in_repo("opt", "\"%1\"", "\"/opt/x\""),
                in_repo("dot-bg", "null", "\"~/dotfiles\""),
                in_repo("dot-a", "\"%2\"", "\"~/dotfiles\""),
                in_repo("dot-b", "\"%3\"", "\"~/dotfiles\""),
                in_repo("none", "\"%4\"", "null"),
                in_repo("home", "\"%5\"", "\"~\""),
                in_repo("work", "\"%6\"", "\"~/Work\""),
            ]
            .concat(),
        )
        .unwrap();
        let ids: Vec<String> = listed(sessions, Some("%3")).into_iter().map(|s| s.id).collect();
        assert_eq!(ids, ["home", "dot-b", "dot-a", "dot-bg", "work", "opt", "none"]);
    }

    #[test]
    fn leaves_out_a_session_that_does_not_parse() {
        let sessions = json::parse_stream(
            &[session("a", "null", "cli"), "{\"session_id\":\"b\"}".into()].concat(),
        )
        .unwrap();
        let ids: Vec<String> = listed(sessions, None).into_iter().map(|s| s.id).collect();
        assert_eq!(ids, ["a"]);
    }
}
