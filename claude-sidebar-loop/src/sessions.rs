//! This host's live Claude sessions, read as `claude-sessions` reads them,
//! which stays the reference and the remote hosts' source: the same
//! sessions, the same left out, the same pane for each, and the same JSON
//! for each, field for field. Change one, change both.
//!
//! They part only over files the hooks never write, where `claude-sessions`'
//! jq fails and this reads on: an unparsable context file, a state with no
//! `session_id`, or a state that is no object, where the jq prints nothing
//! or stops checking the files after it, are each passed over here. A state
//! file holding several JSON values is listed once per value here, and
//! more often by the jq. State files are taken in byte order, where bash
//! collates: the two agree on the UUIDs they are named for.

use crate::gather;
use crate::json::{self, Value};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Where this host's sessions are kept.
pub struct Sources {
    /// the state files `claude-session-state` and `claude-statusline` keep:
    /// `<id>.json`, `<id>.context.json` and `<id>.agents/`
    pub state: PathBuf,
    /// Claude Code's own records, `<pid>.json`
    pub records: PathBuf,
    /// HOME, which a session's `repo` names `~`
    pub home: PathBuf,
}

impl Sources {
    /// as `claude-sessions` finds them, from HOME and XDG_STATE_HOME
    pub fn from_env() -> Sources {
        let home = PathBuf::from(std::env::var_os("HOME").unwrap_or_default());
        let state_home = std::env::var_os("XDG_STATE_HOME")
            .filter(|v| !v.is_empty())
            .map_or_else(|| home.join(".local/state"), PathBuf::from);
        Sources {
            state: state_home.join("claude-sessions"),
            records: home.join(".claude/sessions"),
            home,
        }
    }
}

/// Claude Code's own record of a session.
struct Record {
    name: Value,
    /// a pre-warmed spare, which a `claude daemon` keeps idle for its next
    /// background job and which has nothing to attach to
    spare: bool,
    /// an interactive session, rather than a background job
    interactive: bool,
    /// a background job's id, which Claude Code also gives it as its name
    job_id: Value,
    /// an interactive session's: the id of the background job it parked its
    /// conversation as
    parked_job_id: Value,
    pid: Value,
}

/// Claude Code's records, by session id. Empty where it keeps none, and then
/// a session's liveness is its state file's pid.
type Records = HashMap<String, Record>;

/// {pane id: what it holds}, as `claude-sessions` keys its jq objects
type ByPane = HashMap<String, Value>;

/// A pane on the default tmux server.
struct Pane {
    id: String,
    /// its @claude_attach, the session `claude-sidebar attach` opened in it;
    /// null where tmux printed no field for it
    tag: Value,
    runs_claude: bool,
    /// its window carries @claude_wait: news nobody has looked at
    waiting: bool,
    /// with Claude Code's status glyph taken off
    title: Value,
}

/// every live session, in the order of their state files, each its state
/// file plus `name`, `subagents`, `context`, `repo` and `tmux_pane`, as
/// `claude-sessions` prints them; removes a dead session's files
pub fn read(sources: &Sources) -> Vec<Value> {
    if !sources.state.is_dir() {
        return Vec::new();
    }
    let records = records(&sources.records);
    let sessions = live_sessions(&sources.state, &records, &sources.home);
    if sessions.is_empty() {
        return sessions;
    }
    let panes = panes();
    let shows = shows(&panes, &sessions, &records);
    sessions
        .into_iter()
        .filter(|session| !client_of_another(session, &shows, &records))
        .map(|mut session| {
            let pane = first_pane_showing(&session, &panes, &shows);
            let unseen = unseen(&pane, &panes);
            session.set("tmux_pane", pane);
            session.set("unseen", Value::Bool(unseen));
            session
        })
        .collect()
}

/// whether the window of `pane`, null for none, carries @claude_wait
fn unseen(pane: &Value, panes: &[Pane]) -> bool {
    panes
        .iter()
        .any(|p| p.waiting && pane.as_str() == Some(p.id.as_str()))
}

/// whether `session` is an interactive client whose pane shows another
/// session, which is left out: the pane is the other entry now
fn client_of_another(session: &Value, shows: &ByPane, records: &Records) -> bool {
    let id = session.get("session_id");
    let shown = or(at(shows, session.get("tmux_pane")).clone(), id.clone());
    record(records, id).is_some_and(|r| r.interactive) && &shown != id
}

/// the first pane showing `session`, its recorded one first; null for none
fn first_pane_showing(session: &Value, panes: &[Pane], shows: &ByPane) -> Value {
    let id = session.get("session_id");
    std::iter::once(session.get("tmux_pane").clone())
        .chain(panes.iter().map(|p| Value::String(p.id.clone())))
        .find(|pane| pane != &Value::Null && at(shows, pane) == id)
        .unwrap_or(Value::Null)
}

/// what `by_pane` holds for `pane`, null for none or no pane
fn at<'a>(by_pane: &'a ByPane, pane: &Value) -> &'a Value {
    pane.as_str()
        .and_then(|pane| by_pane.get(pane))
        .unwrap_or(&Value::Null)
}

/// session `id`'s record, if it has one
fn record<'a>(records: &'a Records, id: &Value) -> Option<&'a Record> {
    records.get(id.as_str()?)
}

/// `a // b`: `a` unless it is null or false
fn or(a: Value, b: Value) -> Value {
    if a.truthy() { a } else { b }
}

/// every pane on the default server, even from inside the sidebar's frame
fn panes() -> Vec<Pane> {
    let listed = gather::output(
        Command::new("tmux")
            .args([
                "list-panes",
                "-a",
                "-F",
                "#{pane_id}\t#{@claude_attach}\t#{pane_current_command}\t#{@claude_wait}\t#{pane_title}",
            ])
            .env_remove("TMUX")
            .env_remove("TMUX_PANE"),
    );
    input_lines(&listed).map(Pane::parse).collect()
}

impl Pane {
    /// a line of the list above, its title with Claude Code's status glyph
    /// (idle ✳, working ◐ ◑) taken off
    fn parse(line: &str) -> Pane {
        let fields: Vec<&str> = line.split('\t').collect();
        let title = fields
            .get(4..)
            .map(|rest| rest.join("\t"))
            .unwrap_or_default();
        let title = ["✳ ", "◐ ", "◑ "]
            .iter()
            .find_map(|glyph| title.strip_prefix(glyph))
            .unwrap_or(&title)
            .to_string();
        Pane {
            id: fields[0].to_string(),
            tag: fields
                .get(1)
                .map_or(Value::Null, |t| Value::String(t.to_string())),
            runs_claude: fields.get(2) == Some(&"claude"),
            waiting: fields.get(3).is_some_and(|wait| !wait.is_empty()),
            title: Value::String(title),
        }
    }
}

/// the lines of `text` as jq's `-R` reads them: a last line with no newline
/// is one, and nothing after a final newline is
fn input_lines(text: &str) -> impl Iterator<Item = &str> {
    text.strip_suffix('\n')
        .unwrap_or(text)
        .split('\n')
        .filter(move |_| !text.is_empty())
}

/// Claude Code's own records (`<pid>.json`), keyed by session id, since
/// their pid need not be the one the hooks record; a later record of an id
/// over an earlier one. None at all if one cannot be parsed or is no object
/// naming its session id in a string, as `claude-sessions`' jq then fails;
/// a file that cannot be read is passed over
fn records(dir: &Path) -> Records {
    let mut text = String::new();
    for path in json_files(dir) {
        // cat skips a file it cannot read
        if let Ok(record) = fs::read_to_string(path) {
            text.push_str(&record);
        }
    }
    let Ok(values) = json::parse_stream(&text) else {
        return Records::new();
    };
    values
        .iter()
        .map(|record| match (record, record.get("sessionId")) {
            (Value::Object(_), Value::String(id)) => Some((id.clone(), Record::from(record))),
            // where `claude-sessions`' map({(.sessionId): ...}) fails
            _ => None,
        })
        .collect::<Option<Records>>()
        .unwrap_or_default()
}

impl From<&Value> for Record {
    fn from(record: &Value) -> Record {
        Record {
            name: record.get("name").clone(),
            spare: record.get("spare") == &Value::Bool(true),
            interactive: record.get("kind").as_str() == Some("interactive"),
            job_id: record.get("jobId").clone(),
            parked_job_id: record.get("parkedJobId").clone(),
            pid: record.get("pid").clone(),
        }
    }
}

/// the `*.json` files in `dir`, as bash globs them, in byte order: the
/// order bash's collation gives the session ids and pids they are named for
fn json_files(dir: &Path) -> Vec<PathBuf> {
    let Ok(entries) = fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut names: Vec<String> = entries
        .flatten()
        .filter_map(|e| e.file_name().into_string().ok())
        .filter(|name| name.ends_with(".json") && !name.starts_with('.'))
        .collect();
    names.sort();
    names.into_iter().map(|name| dir.join(name)).collect()
}

/// the live sessions' state files, each with `name`, `subagents`,
/// `context` and `repo` added, less the spares; a dead session's files are removed.
/// A session is live while its pid is: the record's where there are
/// records, since a background job's hooks run under the `claude daemon`,
/// so its state file can hold the daemon's pid, which outlives the job; a
/// session missing from the records has ended
fn live_sessions(dir: &Path, records: &Records, home: &Path) -> Vec<Value> {
    let mut sessions = Vec::new();
    for path in json_files(dir) {
        let name = path.file_name().unwrap_or_default().to_string_lossy();
        let Some(id) = name
            .strip_suffix(".json")
            .filter(|id| !id.ends_with(".context"))
        else {
            continue;
        };
        let id = id.to_string();
        let text = fs::read_to_string(&path).unwrap_or_default();
        // jq reads no further than a file it cannot parse
        let Ok(states) = json::parse_stream(&text) else {
            break;
        };
        // jq passes over a state that is no object
        for mut state in states.into_iter().filter(|s| matches!(s, Value::Object(_))) {
            if !alive(liveness_pid(&state, &id, records)) {
                remove_session(dir, &id);
                continue;
            }
            let own = record(records, state.get("session_id"));
            if own.is_some_and(|r| r.spare) {
                continue;
            }
            add_fields(&mut state, dir, &id, own, home);
            sessions.push(state);
        }
    }
    sessions
}

/// the pid session `id` lives by: its record's where there are records,
/// else its state file's
fn liveness_pid<'a>(state: &'a Value, id: &str, records: &'a Records) -> &'a Value {
    if records.is_empty() {
        state.get("pid")
    } else {
        records.get(id).map_or(&Value::Null, |r| &r.pid)
    }
}

/// removes session `id`'s state file, context file and subagents
fn remove_session(dir: &Path, id: &str) {
    for name in [
        format!("{id}.json"),
        format!("{id}.context.json"),
        format!("{id}.agents"),
    ] {
        remove(&dir.join(name));
    }
}

/// adds `name` (its record's, else its status line's), `subagents`,
/// `context` and `repo` (null for no cwd) to session `id`'s `state`
fn add_fields(state: &mut Value, dir: &Path, id: &str, record: Option<&Record>, home: &Path) {
    let context = context(dir, id);
    let own_name = record.map_or(Value::Null, |r| r.name.clone());
    let repo = match state.get("cwd").as_str() {
        Some(cwd) if !cwd.is_empty() => Value::String(repo(cwd, home)),
        _ => Value::Null,
    };
    state.set("name", or(own_name, context.get("name").clone()));
    state.set("subagents", Value::Number(subagents(dir, id).to_string()));
    state.set("context", context);
    state.set("repo", repo);
}

/// the repo root directory `cwd` is in, as `claude-sidebar` names tmux
/// sessions: a linked worktree's main checkout, a subdirectory's top level,
/// and `cwd` itself outside a repo; `home` as `~`. Read from the .git files
/// rather than asked of git, which this would run for every session every
/// second
fn repo(cwd: &str, home: &Path) -> String {
    let mut dir = Path::new(cwd);
    let root = loop {
        let git = dir.join(".git");
        if git.is_dir() {
            break dir.to_path_buf();
        }
        if git.is_file() {
            // a worktree's .git names its gitdir in its main checkout's .git;
            // a submodule's, one under its superproject's, which is no
            // checkout
            let text = fs::read_to_string(&git).unwrap_or_default();
            let gitdir = text.split_whitespace().nth(1).unwrap_or_default();
            break match gitdir.split_once("/.git/worktrees/") {
                Some((main, _)) if main.starts_with('/') => PathBuf::from(main),
                _ => dir.to_path_buf(),
            };
        }
        match dir.parent() {
            Some(parent) => dir = parent,
            None => break PathBuf::from(cwd),
        }
    };
    match root.strip_prefix(home) {
        Ok(rest) if rest.as_os_str().is_empty() => "~".into(),
        Ok(rest) => format!("~/{}", rest.display()),
        Err(_) => root.display().to_string(),
    }
}

/// whether process `pid` is running, as `kill -0` tells; false for no pid
fn alive(pid: &Value) -> bool {
    unsafe extern "C" {
        fn kill(pid: i32, signal: i32) -> i32;
    }
    let pid = match pid {
        Value::Number(n) | Value::String(n) => n.parse::<i32>().ok(),
        _ => None,
    };
    // SAFETY: signal 0 only checks that the process exists and may be
    // signalled
    pid.is_some_and(|pid| unsafe { kill(pid, 0) } == 0)
}

/// removes the file or directory at `path`, as `rm -rf`
fn remove(path: &Path) {
    if path.is_dir() && !path.is_symlink() {
        let _ = fs::remove_dir_all(path);
    } else {
        let _ = fs::remove_file(path);
    }
}

/// session `id`'s status line record, or null
fn context(dir: &Path, id: &str) -> Value {
    fs::read_to_string(dir.join(format!("{id}.context.json")))
        .ok()
        .and_then(|text| json::parse_stream(&text).ok())
        .and_then(|mut values| values.pop())
        .unwrap_or(Value::Null)
}

/// session `id`'s running subagents: the files in its `.agents`, as a glob
/// counts them
fn subagents(dir: &Path, id: &str) -> usize {
    fs::read_dir(dir.join(format!("{id}.agents"))).map_or(0, |entries| {
        entries
            .flatten()
            .filter(|e| !e.file_name().as_encoded_bytes().starts_with(b"."))
            .count()
    })
}

/// {pane id: the session it shows}. A tagged pane shows its tag, unless that
/// session has a name and the title of the pane names another one: an attach
/// client titles its pane with the title of the conversation, which a
/// resumed job shares with the session it came from, so for a job with no
/// name of its own the title cannot tell the two apart. An untagged Claude
/// pane shows the session its title names, else the one the hooks recorded
/// in it, or the job either parked. The tag before the record, since a
/// `claude attach` client runs its own hooks and records its interactive
/// session in the tagged pane too. A
/// tag naming no live session counts as none: /clear in an attached job
/// gives its conversation a new id, and the pane goes on showing it
fn shows(panes: &[Pane], sessions: &[Value], records: &Records) -> ByPane {
    // the session the hooks recorded in each pane, the last of several
    let recorded: ByPane = sessions
        .iter()
        .filter_map(|s| {
            Some((
                s.get("tmux_pane").as_str()?.to_string(),
                s.get("session_id").clone(),
            ))
        })
        .collect();
    // the one session a title names, by either of its names
    let titled = |pane: &Pane| -> Value {
        let named: Vec<&Value> = sessions
            .iter()
            .filter(|s| s.get("name") == &pane.title || s.get("context").get("name") == &pane.title)
            .map(|s| s.get("session_id"))
            .collect();
        match named.as_slice() {
            [id] if pane.runs_claude => (*id).clone(),
            _ => Value::Null,
        }
    };
    let shown = |pane: &Pane| -> Value {
        let tag_live = sessions.iter().any(|s| s.get("session_id") == &pane.tag);
        let own = names(sessions, &pane.tag, records);
        if !tag_live {
            unparked(
                or(
                    titled(pane),
                    at(&recorded, &Value::String(pane.id.clone())).clone(),
                ),
                sessions,
                records,
            )
        } else if own.is_empty() || own.contains(&&pane.title) {
            pane.tag.clone()
        } else {
            or(titled(pane), pane.tag.clone())
        }
    };
    panes.iter().map(|p| (p.id.clone(), shown(p))).collect()
}

/// the live background job session `id` parked its conversation as, else
/// `id`: starting a new chat while one works parks the working one, and the
/// pane goes on showing it. A parked job since gone counts as none
fn unparked(id: Value, sessions: &[Value], records: &Records) -> Value {
    let Some(job) = record(records, &id)
        .map(|r| &r.parked_job_id)
        .filter(|job| **job != Value::Null)
    else {
        return id;
    };
    sessions
        .iter()
        .map(|s| s.get("session_id"))
        .find(|s| record(records, s).is_some_and(|r| &r.job_id == job))
        .cloned()
        .unwrap_or(id)
}

/// the names session `id` goes by, less the default Claude Code gives a
/// background job (its job id), which is no name at all
fn names<'a>(sessions: &'a [Value], id: &Value, records: &Records) -> Vec<&'a Value> {
    let job = record(records, id).map_or(&Value::Null, |r| &r.job_id);
    sessions
        .iter()
        .filter(|s| s.get("session_id") == id)
        .flat_map(|s| [s.get("name"), s.get("context").get("name")])
        .filter(|name| *name != &Value::Null && *name != job)
        .collect()
}
