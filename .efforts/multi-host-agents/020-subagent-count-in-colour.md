## The sidebar's subagent count in the working colour

**Category:** enhancement
**Summary:** An entry's `+N` running-subagent count is drawn in the sidebar's working colour, so a session with subagents at work stands out even while it is idle itself
**Status:** ready-for-agent
**blocked-by:** []

**Current behavior:**
`claude-sidebar` ends an entry's state line with `+N` when the session has N
subagents running, counted from the `SubagentStart` / `SubagentStop` hooks by
`claude-session-state`. It takes the line's colour: plain on a numbered entry,
dimmed on a background one. Two characters at the end of a line, it is easy
to miss, most of all on an idle session whose subagents are still working.

**Desired behavior:**
- `+N` is painted orange, the colour Clawd takes for `working`, on every
  entry, a dimmed background one included
- The rest of the line, and the entry's layout, are unchanged
- An entry with no running subagents shows nothing, as now

The Maintainer chose colour alone over a glyph or a list of the subagents.

**Key interfaces:**
- `claude-sidebar`'s jq row builder: `clawd($state)`'s colour map and the
  `paint` helper, and the dim prefix a background entry carries
- `claude-sessions`' `subagents` count, unchanged

**Acceptance criteria:**
- [ ] On the Mac, an entry with running subagents shows `+N` in orange,
      idle, working and background entries alike
- [ ] An entry after it in the list keeps its own colours
- [ ] The same on a mgmt-1 entry, once mgmt-1 has pulled

**Out of scope:**
- Whether the count is right; it comes from the hooks as now
- Naming or listing the subagents
