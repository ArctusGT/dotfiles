## One bar on screen under the Mac's mgmt-1 session

**Category:** enhancement
**Summary:** While `ssh-tmux` shows mgmt-1 in the Mac's tmux, the Mac hides its own bar for that session, leaving mgmt-1's bar, with mgmt-1's windows, as the only one on screen
**Status:** ready-for-closure
**blocked-by:** [multi-host-agents/004-cross-host-agent-view]

**Current behavior:**
`ssh-tmux` runs mgmt-1's whole tmux inside a Mac pane. The remote status bar
draws inside that pane, offset from the Mac's own, so two bars are on screen.

**Desired behavior:**
While mgmt-1's tmux is reached from the Mac, the Mac session holding it draws
no bar. The remote's bar is the one on screen, with the remote's windows, and
`ctrl+left`/`ctrl+right` move between them. The Mac's sidebar (slice 004)
reaches every session on both hosts. Leaving the relay, by detaching, a
closed window or ctrl-c, brings the Mac's bar back.

**Key interfaces:**
- `ssh-tmux`, which knows it is the relay and tags its window `@ssh_tmux`
- The Mac's per-session `status` option
- The `claude-tmux-status` relay bell, which passes through the remote client
  and keeps marking the Mac window

**Acceptance criteria:**
- [x] Opening `mgmt-1` from the sesh picker shows mgmt-1's bar and not the
      Mac's
- [x] `ctrl+left`/`ctrl+right` there move between mgmt-1's windows
- [x] The remote session chooser still switches between mgmt-1's sessions in
      that pane
- [x] Reconnecting after a dropped connection keeps the Mac's bar hidden
- [x] Closing the window, or ctrl-c while it waits to reconnect, restores the
      Mac session's bar and ends the reconnect loop
- [x] A waiting mgmt-1 session still marks the Mac's `mgmt-1` window
- [x] The Readme describes what the remote shows under the Mac

**Out of scope:**
- How mgmt-1's tmux looks when reached other than through `ssh-tmux`
- Hosts other than mgmt-1

## Comments

- The slice first asked for mgmt-1's bar to go. The Maintainer chose to hide
  the Mac's instead: mgmt-1's bar carries mgmt-1's windows, and the change
  stays on the Mac. The remote sidebar it also asked to hide was workmux's,
  gone with slice 004.
- `code-review` on b7a8056 and 41283f6; fixed in f03d8e8: `claude-tmux-status`
  rang every tmux* client as a relay, which since tmux-frame includes the
  Mac's own frame client, so it now rings only the client ssh-tmux records as
  @relay_client; a stale claim in its header; both headers short of what the
  scripts read and change.
- 41283f6, ringing the relay for a session with no pane, went beyond "still
  marks": such a session never marked the window before. It answers the
  Maintainer's test of this criterion, and sits beside slice 008.
- Left open from that review, for the Maintainer:
  - Two ssh-tmux windows in one Mac session: the first to exit brings the bar
    back while the other runs. One sesh entry per host keeps this from arising.
  - Other windows in the relay's Mac session lose the bar too; the sesh
    `mgmt-1` session holds only the relay.
  - A relay window moved to another session leaves the old session bar-less
    and the new one with its bar, until ssh-tmux exits.
