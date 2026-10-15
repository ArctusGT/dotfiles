## Remote tmux goes bare under the Mac

**Category:** enhancement
**Summary:** Viewed through `ssh-tmux`, mgmt-1's tmux shows no status bar and no sidebar, leaving the Mac's tmux as the only chrome on screen
**Status:** ready-for-agent
**blocked-by:** [multi-host-agents/004-cross-host-agent-view]

**Current behavior:**
`ssh-tmux` runs mgmt-1's whole tmux inside a Mac pane. The remote status bar
draws inside that pane, offset from the Mac's own, and the remote sidebar
takes a second strip of width beside the Mac's.

**Desired behavior:**
While mgmt-1's tmux is reached from the Mac, it draws no status bar and no
sidebar. The pane shows only the remote window's panes. The Mac's sidebar
(slice 004) is the one place to see and reach mgmt-1's sessions. Moving between
mgmt-1's sessions and windows still works from the keyboard on the remote:
its session chooser and window keys.

**Key interfaces:**
- `ssh-tmux`, which knows it is the relay and can tell the remote tmux so
- The remote tmux's per-session or per-client status setting
- The `claude-tmux-status` relay bell, which passes through the remote client
  and keeps marking the Mac window

**Acceptance criteria:**
- [ ] Opening `mgmt-1` from the sesh picker shows no remote status bar and no
      remote sidebar
- [ ] The remote session chooser still switches between mgmt-1's sessions in
      that pane
- [ ] Reconnecting after a dropped connection comes back bare
- [ ] A waiting mgmt-1 session still marks the Mac's `mgmt-1` window
- [ ] The Readme describes what the remote shows under the Mac

**Out of scope:**
- How mgmt-1's tmux looks when reached other than through `ssh-tmux`
- Hosts other than mgmt-1
