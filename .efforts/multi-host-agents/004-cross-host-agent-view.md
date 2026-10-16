## One view of agents across hosts

**Category:** enhancement
**Summary:** A frame on the Mac keeps a sidebar of every Claude session on the Mac and on mgmt-1 beside the usual tmux, and shows any of them on the right; it replaces workmux
**Status:** ready-for-closure
**blocked-by:** [multi-host-agents/006-session-state-files]

**Current behavior:**
Each host runs a workmux sidebar listing its own agents, one entry per
worktree. Several sessions in one repo show as one, and sessions outside tmux
do not show at all. A mgmt-1 session is visible from the Mac only inside the
`ssh-tmux` window, through a second, nested sidebar. workmux is used for that
sidebar and for nothing else.

**Desired behavior:**
One sidebar on the Mac lists every session from both hosts' state files
(slice 006). Each row shows host, repo, session name, state, time in that
state, running subagents and context percentage. A row whose context passes a
threshold is marked, as the signal to hand off and clear.

- Selecting a Mac session in tmux jumps to its pane
- Selecting a mgmt-1 session in tmux switches the Mac to that host's
  `ssh-tmux` session, then moves mgmt-1's one client to the session's pane
- A session with no tmux pane, such as one from the desktop app, is listed
  and not jumpable
- An unreachable host shows as unreachable, not as an empty list
- A stale state file, whose pid is gone, is not listed

workmux comes out entirely: its stow package, its Claude hooks, its tmux
bindings and its sidebar, on both hosts.

The sidebar lives in a frame: a tmux server of its own with no bar and no
prefix, the sidebar on the left and a client of the usual tmux on the right.
The sidebar stays on screen while sessions change on the right, by its own
selection or by `prefix f` there. Sessions are grouped by host, and those with
a pane are numbered for a hotkey. mgmt-1 stays a window in the Mac's tmux,
reached through `ssh-tmux` as now.

The sidebar is a short shell script driving fzf with a timed reload, not a
bespoke TUI.

**Key interfaces:**
- The state-file directory from slice 006, read locally and over
  `ssh sysadmin@mgmt-1`, which logs in without a prompt
  - A pid can carry several sessions: a `claude daemon` hosts each of its
    sessions under its own pid, so rows are keyed by session id, never by pid
  - Desktop-app sessions write no context file, so their rows have no
    context percentage
- The hosts to include, as configuration rather than code
- The `@ssh_tmux` window option `ssh-tmux` sets, mapping a host to the Mac
  window holding its session
- mgmt-1 has one client from the Mac (slice 005), so a jump there targets that
  client by name

**Acceptance criteria:**
- [x] The sidebar lists every session that has a live state file on the Mac
      and on mgmt-1, including several in one repo
- [x] Each row shows host, repo, name, state, time in state, subagents and
      context percentage, and refreshes within a few seconds of a change
- [x] Rows past the context threshold are marked; the threshold is one setting
- [x] Selecting a Mac tmux session, by click, Enter or its number's hotkey,
      shows its pane in the frame's right pane and focuses it
- [x] Selecting a mgmt-1 tmux session puts that pane on screen in the Mac's
      `mgmt-1` session, in the frame's right pane
- [x] Desktop-app sessions are listed and selecting one does nothing harmful
- [x] With mgmt-1 unreachable, its rows are replaced by one unreachable row
      and the Mac's rows still update
- [x] The frame shows no bar of its own, and passes every tmux key to the
      right pane, where `prefix f` switches only that pane
- [x] Entries are grouped by host
- [x] workmux's package, hooks and bindings are gone from the repo, and
      workmux's hooks are gone from both hosts' Claude settings
- [x] The Readme lists the sidebar in place of workmux

**Out of scope:**
- Hosts other than the Mac and mgmt-1 until the Maintainer names them
- Starting, stopping or messaging sessions; this is a view and a jump
- Hiding mgmt-1's own status bar and sidebar (slice 007)
- Listing subagents individually; the row carries a count

## Comments

- "as workmux's did": no key was ever bound for workmux's sidebar; it ran as
  a command.
- `code-review` on ef77750..390e702; fixed in 5e87d6c: a quiet host behind a
  live ssh master froze every host's rows (now ServerAlive, 4s), a stale
  refresh period in the Readme, and the sidebar's duplicated usage line.
- Left open from that review, for the Maintainer:
  - The cache is one fixed directory, so a second `claude-sidebar` run
    outside the frame shares it, and its exit removes the frame sidebar's
    cache and socket. Nothing runs a second one today.
  - A reused pid keeps a dead session listed, through `claude-sessions`'
    `kill -0` (slice 006's reader, not this diff).
  - Both hosts keep a `workmux` directory in `~/.claude/plugins/cache`,
    left by `claude plugin uninstall`; nothing loads it.
