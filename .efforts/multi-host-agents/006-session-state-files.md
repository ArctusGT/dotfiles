## Each Claude session reports its own state

**Category:** enhancement
**Summary:** Every Claude session on a host keeps one small state file current, carrying its state, its place in tmux and its context-window use
**Status:** ready-for-agent
**blocked-by:** none

**Current behavior:**
No source lists every Claude session on a host:

- workmux keeps one entry per worktree, so a repo running several sessions
  shows as one. It sees only sessions in tmux panes.
- `claude agents --json` reads Claude's internal session records and drops
  live sessions that lack one, including both tmux-hosted sessions on mgmt-1.
- tmux sees only what runs in tmux, and misses the desktop app's sessions over
  SSH and background (`--bg`) sessions.

Context-window use is visible only inside each session's own UI. The
`claude-tmux-status` hook marks a waiting session's tmux window and rings its
bell, and holds no other state.

**Desired behavior:**
Each host keeps one directory of per-session state files, written only by the
user-level Claude settings: the hooks plus a `statusLine` command. A file
appears when a session starts and disappears when it ends. It holds:

- session id, pid, cwd, and `$TMUX_PANE` when the session runs in tmux
- state (working, waiting on the Maintainer, idle) and when it entered that
  state
- the number of subagents currently running
- context-window use from the status line input: tokens used, window size,
  percentage; and the session's cost

Each Claude session shows its context percentage in its own status line too.

A reader takes the whole directory in one read, locally or over ssh, and needs
no access to Claude's internals. Files left behind by a session that died
without an end hook are recognisable as stale: their pid is gone.

**Key interfaces:**
- The hook events in the installed Claude Code, for session start and end,
  prompt submit, stop, notification, and subagent start and stop. Confirm
  their names against the running version before wiring them; a docs summary
  gave names that disagree with the ones the settings already use
- The `statusLine` stdin JSON: `context_window.used_percentage`,
  `context_window.context_window_size`, `context_window.current_usage`,
  `cost.total_cost_usd` (https://code.claude.com/docs/en/statusline)
- The state-file format, which slice 004 reads: one JSON object per session,
  named by session id
- `claude-tmux-status`'s wait and clear behaviour, which keeps working

**Acceptance criteria:**
- [ ] On the Mac and on mgmt-1, every live Claude session in a tmux pane has a
      state file, including several sessions in one repo
- [ ] Measured and recorded in the closing commit: which of the desktop app's
      SSH sessions and background sessions write a state file, and which carry
      context-window fields
- [ ] A session's state reads `working` while it runs a turn, `waiting` while a
      permission prompt or question is open, and `idle` after it stops
- [ ] Starting a subagent raises the session's subagent count and its finishing
      lowers it
- [ ] Context used, window size and percentage match what the session's own
      `/context` reports, within one turn
- [ ] Ending a session removes its file; killing one leaves a file whose pid
      no longer exists
- [ ] Each session's status line shows its context percentage
- [ ] The marker and bell from `claude-tmux-status` still work on both hosts

**Out of scope:**
- Any view or sidebar reading the files (slice 004)
- Rate-limit or plan-quota tracking across sessions
- Sessions on hosts other than the Mac and mgmt-1
