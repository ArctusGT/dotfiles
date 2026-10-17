## A pane's entry follows the session it shows

**Category:** bug
**Summary:** When a Claude window switches to another session through Claude Code's own session list, the sidebar's entry for that pane takes on the session now on show, its name, state and context, found from the pane's terminal title
**Status:** ready-for-agent
**blocked-by:** [multi-host-agents/014-names-from-claude-records]

**Current behavior:**
A pane's entry is tied to the session the hooks first recorded in it. After
the Claude client there switches to a background session from Claude Code's
session list (or `←`, the agent view), the entry still shows the client's
own session: on mgmt-1, pane `%4` showed `3623e096`, idle and unnamed, while
the session on screen, `d89a4a47` ("sync dotfiles tmux config"), was working
and listed further down as a dimmed background row. Its state never
changes, so the numbered entry reads idle while work happens.

Slice 008 put this out of scope on the claim that nothing reports the
switch. Claude Code does: it sets the terminal title to the name of the
session on show (`✳ dotfiles-main`), and tmux holds it as `pane_title`.

**Desired behavior:**
When a Claude pane's title names exactly one other live session on the same
host, by the names slice 014 brings, the sidebar shows that session in the
pane's entry: its name, state, time, subagents and context line, numbered
and followed by the cursor like any entry with a pane, and selecting it
shows that pane. This holds for every Claude pane, `@claude_attach` panes
included: on mgmt-1, `%15`, tagged `30b4d761` (cost-recovery-time-logging),
was titled `gitlab` (`633cb9ec`) at one sampling and its own name a few
minutes later, so a tagged pane switches like any other. A tagged background
session whose pane shows another one stays listed as a background row with
no pane, and selecting it opens a fresh attach window rather than reusing
that pane. Only an interactive client's own record is left out while its
pane displays another session. When the title matches no session, more than
one, or a default (`Claude Code`, a host name, `claude agents`), the entry
stays as it is now. Switching back, or to a third session, is followed
within a refresh.

**Key interfaces:**
- `#{pane_title}` from each host's tmux, read with the panes
  `claude-sessions` already lists; the title carries a leading status glyph
  (`✳` when idle, `◐` and `◑` while working, measured in the title code of
  2.1.292 and 2.1.302; static `✳` under tmux, and none from a `claude
  attach` client) to strip before matching
- Session names from Claude Code's per-session records (slice 014)
- The pane-to-session mapping `claude-sessions` already does for
  `@claude_attach` panes (slice 008), which this extends: a title naming
  another session wins over the tag, and `claude-sidebar attach` reuses a
  pane only while it still shows the session
- The title format is Claude Code's, unversioned: measure it on the Mac's
  and mgmt-1's releases, and fall back to today's behaviour when it does
  not parse

**Acceptance criteria:**
- [ ] After switching a Claude window to a background session from Claude
      Code's session list, the pane's entry shows that session's name and
      live state within a refresh, on the Mac and on mgmt-1
- [ ] That session is no longer listed a second time as a background row,
      and the client's own unnamed record is not listed
- [ ] Switching the window back shows the client's own session again
- [x] A pane whose title is ambiguous or a default keeps today's entry
- [x] The Readme's sidebar section says a pane follows the session it shows

**Out of scope:**
- Changing Claude Code's titles, or reading its transcripts
