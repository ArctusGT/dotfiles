## Session names from Claude Code's own records

**Category:** bug
**Summary:** The sidebar shows each session's current name, renames included, from the record Claude Code keeps per session, rather than the status line's last record, which goes stale for any session not on screen
**Status:** ready-for-closure
**blocked-by:** []

**Current behavior:**
`claude-sessions` takes a session's `name` from the status line's last
record, which is written only while the session is drawn. A background
session renamed from Claude Code's session list keeps its old name in the
sidebar (mgmt-1: `gitlab-install-repo-remote-restore` after a rename to
`gitlab`), and a session whose status line never ran shows no name at all,
so the sidebar falls back to its short id (`db661fc7`, which Claude Code
calls `hours-97`).

**Desired behavior:**
Every session's name is the one Claude Code currently holds for it, on the
Mac and on mgmt-1, and a rename shows in the sidebar within a refresh,
whether the session is on screen, in a pane or in the background. The status
line's record stays the source of context use and cost.

**Key interfaces:**
- Claude Code's per-session records, `~/.claude/sessions/<pid>.json`: each
  carries `sessionId`, `name` (with `nameSince`), `jobId`, `kind`, `status`
  and `spare`. Match them by `sessionId`: their pid need not be the one the
  hooks record (this session: 74842 in its record, 74501 in its state file).
  `claude-sessions` already reads them to leave out spares
- `claude agents --json` carries the same names, at about 0.1s per call; the
  records are plain file reads, which suits a once-a-second refresh
- A record's `name` may be a default (the job id, as for a spare, or an
  automatic one); show it as Claude Code does

**Acceptance criteria:**
- [x] A session renamed with `/rename` shows its new name in the sidebar
      within a refresh, on the Mac and on mgmt-1, without being on screen
- [x] A background session renamed from Claude Code's session list shows
      the new name
- [x] A session whose status line never ran shows Claude Code's name for
      it, not its short id
- [x] Context use and cost still come from the status line's record

**Out of scope:**
- Which session a pane shows after switching inside Claude Code
  (slice 015)
- Renaming from the sidebar

**Left for the Maintainer:**
- Criteria 1 and 2 need a live rename: `/rename` in a session that isn't
  on screen, and a rename of a background session from Claude Code's
  session list, each on the Mac and on mgmt-1 (after mgmt-1 pulls). The
  record is re-read on every refresh, and a rename shows in it (on the Mac,
  84e750a9's record carries `formerNames: [Work sliced]` and `nameSource:
  user`), but no rename was watched landing in the sidebar.
- A session with a derived default name now shows that name, as Claude
  Code's session list does, where the status line's summary title showed
  before: 8db3d82a reads `obsidian-thislists-29`, no longer `Reddit links
  rendering`.
- Three reviews noted that `jq -s` drops every record for a refresh if any
  one fails to parse (a write caught half-done, say). No torn record has
  been seen and nobody has measured whether Claude Code writes them
  atomically, so there is no guard.
