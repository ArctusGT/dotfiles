//! The entry numbers the technician pinned to repos, which `claude-sidebar
//! pin` writes: `${XDG_STATE_HOME:-~/.local/state}/claude-sidebar/pins`, a
//! line a pin of its number, the repo's ssh host (empty for this one) and
//! the repo as `claude-sessions` gives it, tab-separated.

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// {(ssh host, repo): number}
#[derive(Default)]
pub struct Pins(HashMap<(String, String), usize>);

impl Pins {
    /// the pins file's, none where there is no file; a line that is not a
    /// number 1 to 9 and two fields is passed over
    pub fn read() -> Pins {
        Pins::parse(&fs::read_to_string(path()).unwrap_or_default())
    }

    pub fn parse(text: &str) -> Pins {
        let pins = text.lines().filter_map(|line| {
            let mut fields = line.split('\t');
            let number = fields.next()?.parse().ok().filter(|n| (1..=9).contains(n))?;
            let host = fields.next()?.to_string();
            let repo = fields.next()?.to_string();
            Some(((host, repo), number))
        });
        Pins(pins.collect())
    }

    /// the number pinned to `repo` on ssh host `host`, if any
    pub fn number(&self, host: &str, repo: &str) -> Option<usize> {
        self.0.get(&(host.to_string(), repo.to_string())).copied()
    }

    /// whether a pin holds `number`, whether or not its repo has an entry
    pub fn holds(&self, number: usize) -> bool {
        self.0.values().any(|&n| n == number)
    }
}

fn path() -> PathBuf {
    let state_home = std::env::var_os("XDG_STATE_HOME")
        .filter(|v| !v.is_empty())
        .map_or_else(
            || PathBuf::from(std::env::var_os("HOME").unwrap_or_default()).join(".local/state"),
            PathBuf::from,
        );
    state_home.join("claude-sidebar/pins")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_pins_and_passes_over_lines_it_cannot() {
        let pins = Pins::parse("3\t\t~/dotfiles\n7\tsysadmin@mgmt-1\t~\n12\t\t~/x\nnope\n4\t\n");
        assert_eq!(pins.number("", "~/dotfiles"), Some(3));
        assert_eq!(pins.number("sysadmin@mgmt-1", "~"), Some(7));
        assert_eq!(pins.number("", "~"), None);
        assert!(pins.holds(7) && !pins.holds(12) && !pins.holds(4));
    }
}
