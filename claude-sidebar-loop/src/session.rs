//! A session as the sidebar draws it, parsed once from the JSON object
//! `claude-sessions` prints. One the hooks wrote parses whole; one with a
//! field missing or of the wrong type is an error the gather logs and leaves out.

use crate::json::Value;

/// what `claude-session-state` says a session is doing; the only states it
/// writes
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum State {
    Idle,
    Working,
    Waiting,
}

impl State {
    fn from_word(word: &str) -> Result<State, String> {
        match word {
            "idle" => Ok(State::Idle),
            "working" => Ok(State::Working),
            "waiting" => Ok(State::Waiting),
            other => Err(format!("state is {other}")),
        }
    }

    /// the word the sidebar shows, as the hooks write it
    pub fn word(self) -> &'static str {
        match self {
            State::Idle => "idle",
            State::Working => "working",
            State::Waiting => "waiting",
        }
    }
}

/// a number as the sidebar shows it: the text jq prints for it (`42.50`
/// stays `42.50`), and its value
#[derive(Clone, Debug, PartialEq)]
pub struct Number {
    pub text: String,
    pub value: f64,
}

impl Number {
    /// `value`, which is a number or null; null is none, anything else an
    /// error
    fn optional(value: &Value) -> Result<Option<Number>, String> {
        match value {
            Value::Null => Ok(None),
            number => Number::required(number).map(Some),
        }
    }

    fn required(value: &Value) -> Result<Number, String> {
        match value.as_f64() {
            Some(n) => Ok(Number {
                text: value.text(),
                value: n,
            }),
            None => Err(format!("{value} is no number")),
        }
    }
}

/// one session of a host, as the sidebar lists and draws it
#[derive(Clone, Debug, PartialEq)]
pub struct Session {
    pub id: String,
    /// the pane on its host's tmux showing it; none for a background job, or
    /// a session of a desktop app
    pub pane: Option<String>,
    /// blank where the state file has none
    pub cwd: String,
    /// the repo root `cwd` is in, `~` for HOME; none where there is no cwd,
    /// or the host's claude-sessions predates it
    pub repo: Option<String>,
    /// the name Claude Code or the status line gave it
    pub name: Option<String>,
    pub state: State,
    /// when it entered `state`, in seconds since the epoch
    pub since: f64,
    /// the subagents running
    pub subagents: Number,
    /// the context used, in percent, where the status line has recorded it
    pub used_pct: Option<Number>,
    /// a session of the desktop app, which has no pane to jump to unless
    /// one shows it
    pub desktop: bool,
    /// its pane's window carries @claude_wait: a turn ended or a question
    /// came while nobody was looking. False where the host's output has no
    /// such field
    pub unseen: bool,
}

impl Session {
    /// whether it has subagents running
    pub fn subagents_running(&self) -> bool {
        self.subagents.value > 0.0
    }

    /// the session `object` describes, or why it is none: a field missing
    /// where one is required, or of the wrong type
    pub fn parse(object: &Value) -> Result<Session, String> {
        let field = |key: &str| object.get(key);
        let text = |key: &str| match field(key) {
            Value::String(s) => Ok(s.clone()),
            other => Err(format!("{key} is {other}, not a string")),
        };
        // null for absent, as jq's `.key`
        let optional_text = |key: &str| match field(key) {
            Value::Null => Ok(None),
            _ => text(key).map(Some),
        };
        let state = State::from_word(&text("state")?)?;
        Ok(Session {
            id: text("session_id")?,
            pane: optional_text("tmux_pane")?,
            cwd: optional_text("cwd")?.unwrap_or_default(),
            repo: optional_text("repo")?,
            name: optional_text("name")?,
            state,
            since: Number::required(field("since"))?.value,
            subagents: Number::required(field("subagents"))?,
            used_pct: Number::optional(field("context").get("used_pct"))?,
            desktop: field("entrypoint").as_str() == Some("claude-desktop"),
            unseen: matches!(field("unseen"), Value::Bool(true)),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::json::parse_stream;

    fn parse(text: &str) -> Result<Session, String> {
        Session::parse(&parse_stream(text).unwrap()[0])
    }

    const WHOLE: &str = r#"{"session_id":"a","pid":1,"cwd":"/x/y","tmux_pane":"%3",
        "entrypoint":"cli","state":"waiting","since":1792310933,"name":"n","subagents":2.0,
        "context":{"used_pct":42.50}}"#;

    #[test]
    fn reads_what_the_hooks_write() {
        let session = parse(WHOLE).unwrap();
        assert_eq!(session.pane.as_deref(), Some("%3"));
        assert_eq!(session.state, State::Waiting);
        assert_eq!(session.subagents.text, "2.0");
        assert_eq!(session.used_pct.unwrap().text, "42.50");
        assert!(!session.desktop);
    }

    #[test]
    fn unseen_is_true_only_where_the_host_says_so() {
        assert!(!parse(WHOLE).unwrap().unseen);
        let marked = WHOLE.replace(r#""cwd""#, r#""unseen":true,"cwd""#);
        assert!(parse(&marked).unwrap().unseen);
        let clear = WHOLE.replace(r#""cwd""#, r#""unseen":false,"cwd""#);
        assert!(!parse(&clear).unwrap().unseen);
    }

    #[test]
    fn a_missing_pane_name_context_or_cwd_is_none() {
        let session = parse(
            r#"{"session_id":"a","state":"idle","since":1,"subagents":0,"context":null,
                "tmux_pane":null,"name":null}"#,
        )
        .unwrap();
        assert_eq!((session.pane, session.name, session.used_pct), (None, None, None));
        assert_eq!(session.cwd, "");
    }

    #[test]
    fn a_session_with_a_field_of_the_wrong_kind_is_an_error() {
        for (field, bad) in [
            ("state", r#""asleep""#),
            ("since", r#""soon""#),
            ("subagents", "null"),
            ("session_id", "7"),
            ("tmux_pane", "7"),
        ] {
            let mut broken = parse_stream(WHOLE).unwrap().remove(0);
            broken.set(field, parse_stream(bad).unwrap().remove(0));
            assert!(Session::parse(&broken).is_err(), "{field}");
        }
    }
}
