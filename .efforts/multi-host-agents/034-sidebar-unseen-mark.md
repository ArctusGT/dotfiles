## The sidebar marks a session whose news is unseen

**Category:** enhancement
**Summary:** Each sidebar entry carries the unseen mark the tmux status bar's dots show today, a session whose turn ended or that asked something while its window was not being looked at, read from the same window flag, so the dots can later be retired
**Status:** ready-for-human
**blocked-by:**

**Current behavior:**
`claude-tmux-status wait`, on Claude Code's `Notification` and `Stop`,
sets `@claude_wait` on the session's tmux window unless that window is
being looked at; selecting the window, or a client changing session,
unsets it. The status bar's right side draws a `●` in colour 3 with
`session:window` for every marked window, on every host whose tmux draws
it. The sidebar shows each session's state, and a session whose turn
ended an hour ago unseen reads `idle`, the same as one just looked at, so
the dots are the only place the difference shows.

**Desired behavior:**
- Each entry whose pane's window carries `@claude_wait` shows the
  status bar's mark, a `●` in colour 3, in its text; the Clawd is
  unchanged
- The mark comes from the flag the hooks already keep, read with the
  panes each gather lists; no second store and no new hook
- It clears in the sidebar within a gather of the flag clearing, which
  includes a jump from the sidebar to that session
- Remote sessions carry their host's flag, read by that host's
  `claude-sessions`; a session with no pane, and a host whose
  `claude-sessions` predates the field, show no mark

The Maintainer chose to add the mark first, then a Clawd pose for it,
then retire the status bar's dots.

**Key interfaces:**
- The panes the binary and `claude-sessions` list from the default tmux
  server, and the session record each emits; a user option read in a
  pane's format resolves through its window
- The renderer's typed `Session`
- `@claude_wait` and the hooks that set and unset it

**Acceptance criteria:**
- [x] On a throwaway tmux server with its own TMPDIR, setting and unsetting
      `@claude_wait` on a session's window shows and clears the mark within
      a gather, and the other entries' rows are byte-identical to before
- [x] The binary's local sessions still equal `claude-sessions`' output,
      by the shipped comparison, with the field present in both
- [x] A session without the flag, a session with no pane, and a remote
      host whose output lacks the field all render as today
- [ ] The Maintainer sees the mark beside a session the status bar shows a
      dot for, locally and on mgmt-1, and sees it clear on jumping to it

**Out of scope:**
- A Clawd pose or animation for an unseen session
- Retiring the status bar's dots, `prefix a`, or `claude-tmux-status`
  and its bell
- Changing when the hooks set or unset `@claude_wait`
