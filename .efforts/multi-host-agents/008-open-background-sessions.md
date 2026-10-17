## Open background sessions from the sidebar

**Category:** enhancement
**Summary:** Selecting a background Claude session in the sidebar opens it in a tmux window of its own on its host, reused on later selects, so it becomes a numbered, jumpable entry like any other
**Status:** ready-for-agent
**blocked-by:** [multi-host-agents/004-cross-host-agent-view]

**Current behavior:**
A session a `claude daemon` runs (`--bg`, or one moved to the background from
Claude Code's own session list) has no tmux pane. The sidebar lists it dimmed
as `background`, and selecting it does nothing. Opening it from inside a
running Claude window works, but that window keeps the identity of the
session it started with, so the sidebar neither highlights nor tracks the
session on show.

**Desired behavior:**
Selecting a background entry, by click, Enter or its number's hotkey, opens
that session in a tmux window on the session's own host and shows it in the
frame's right pane. The window runs Claude Code's `attach` for that session
id and is tagged with it. A later select of the same entry jumps to the
tagged window, not a second copy. While the window exists, the entry is
numbered and the cursor follows it like any session with a pane. Closing the
window detaches; the session keeps running in its daemon and the entry goes
back to dimmed.

**Key interfaces:**
- `claude attach <id|name>`: opens a background session in the current
  terminal, on the Mac and on mgmt-1
- A window option carrying the attached session id, read by the sidebar's
  per-host collection alongside `claude-sessions`
- The tmux session the window opens in: the one the frame's right pane shows
  on the Mac, or the one the Mac's relay client is on for a remote host
- A remote open reaches the host over the sidebar's shared ssh connection and
  must not hold up fzf (see the background ssh in a remote jump)

**Acceptance criteria:**
- [ ] Selecting a Mac background entry opens it in a new window in the right
      pane's tmux session and shows it there
- [ ] Selecting a mgmt-1 background entry opens it in a window on mgmt-1 and
      shows it in the Mac's `mgmt-1` session
- [ ] Selecting the same entry again jumps to that window; no second window
      opens
- [ ] An entry with an open window is numbered, reachable by its hotkey, and
      followed by the cursor
- [ ] Closing the window leaves the session running and the entry dimmed again
- [ ] The Readme describes what selecting a background entry does

**Out of scope:**
- Tracking which conversation a Claude window shows after switching inside
  Claude Code's own session list; nothing reports that change
- Starting, stopping or messaging sessions
- Desktop app sessions, which the sidebar leaves out
