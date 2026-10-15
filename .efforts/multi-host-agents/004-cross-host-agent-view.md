## One view of agents across hosts

**Category:** enhancement
**Summary:** One sidebar on the Mac lists every Claude session on the Mac and on mgmt-1, with its context use, and jumps to any of them; it replaces workmux
**Status:** ready-for-agent
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
- [ ] The sidebar lists every session that has a live state file on the Mac
      and on mgmt-1, including several in one repo
- [ ] Each row shows host, repo, name, state, time in state, subagents and
      context percentage, and refreshes within a few seconds of a change
- [ ] Rows past the context threshold are marked; the threshold is one setting
- [ ] Selecting a Mac tmux session focuses its pane
- [ ] Selecting a mgmt-1 tmux session puts that pane on screen in the Mac's
      `mgmt-1` session
- [ ] Desktop-app sessions are listed and selecting one does nothing harmful
- [ ] With mgmt-1 unreachable, its rows are replaced by one unreachable row
      and the Mac's rows still update
- [ ] The sidebar toggles from a tmux key, as workmux's did
- [ ] workmux's package, hooks and bindings are gone from the repo, and
      workmux's hooks are gone from both hosts' Claude settings
- [ ] The Readme lists the sidebar in place of workmux

**Out of scope:**
- Hosts other than the Mac and mgmt-1 until the Maintainer names them
- Starting, stopping or messaging sessions; this is a view and a jump
- Hiding mgmt-1's own status bar and sidebar (slice 007)
- Listing subagents individually; the row carries a count
