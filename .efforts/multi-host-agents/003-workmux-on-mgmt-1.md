## workmux on mgmt-1

**Category:** enhancement
**Summary:** Run workmux and its Claude status plugin on mgmt-1, as on the Mac
**Status:** ready-for-closure
**blocked-by:** [multi-host-agents/002-newer-tmux-on-mgmt-1]

**Current behavior:**
workmux runs on the Mac only. Its global config is the `workmux` stow package,
and the `workmux-status` Claude plugin is enabled in the Mac's user settings.
mgmt-1 keeps its own Claude settings (model and effort level differ from the
Mac's), which carry the `claude-tmux-status` hooks but no plugin. On mgmt-1,
`~/.config` is a symlink into the repo's `nvim/.config`, which the repo
gitignores apart from `nvim/` itself.

**Desired behavior:**
On mgmt-1, `workmux add <name>` in a repo creates a worktree and a window
running Claude beside a shell, and the window's name shows the agent's working,
waiting and done states. `workmux sidebar` lists the agents in mgmt-1's
sessions. From the Mac, both are visible through the `ssh-tmux` window. The
global config is the same file the Mac uses.

**Key interfaces:**
- workmux's Linux install: a release binary or `cargo install`, into the
  user's own `bin`
- The `workmux-status` plugin, installed into mgmt-1's own Claude user
  settings without replacing its model, effort level or hooks
- The `workmux` stow package, which must land at
  `~/.config/workmux/config.yaml` on mgmt-1 despite `~/.config` resolving
  into `nvim/.config`; the result must not leave untracked files that a
  stray `git add -A` would publish

**Acceptance criteria:**
- [x] `workmux --version` works on mgmt-1 from an interactive shell and
      over `ssh mgmt-1`
- [x] `workmux config path` on mgmt-1 resolves to the stowed config, and
      `git status` in mgmt-1's dotfiles shows no untracked files from it
- [x] mgmt-1's Claude settings enable `workmux-status` and still hold their
      model, effort level and `claude-tmux-status` hooks
- [x] `workmux add` in a mgmt-1 repo opens a worktree window, and its name
      shows a status icon once Claude runs
- [x] `workmux sidebar` on mgmt-1 lists that agent, seen from the Mac through
      the `ssh-tmux` window
- [x] The Readme's prerequisites no longer say workmux is macOS-only

**Out of scope:**
- Combining the two hosts' agents into one view (slice 004)
- Removing `claude-tmux-status`; it stays the only signal that reaches the Mac
  from mgmt-1
