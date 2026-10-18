## The Rust binary renders the sidebar's frames

**Category:** enhancement
**Summary:** The sidebar's frames are rendered inside the Rust binary from the gathered sessions, in place of `format`'s jq and `draw`'s perl split, byte-identical to today's rows
**Status:** ready-for-agent
**blocked-by:** [multi-host-agents/030-rust-posts-frames-to-fzf]

**Current behavior:**
Each gather, `draw` runs `format`, one jq program, over the tagged
sessions for all 48 frames, then a perl script that splits its output
into `rows.N` and writes `repeats`. In the live sidebar these cost 1.8%
and 1.0% of a core, plus a jq and a perl spawn each second.

**Desired behavior:**
- The binary renders every frame from the tagged sessions in-process:
  each entry's fields, number, the text right of the picture, the
  context line with its gradient and warn colour, dimming of a session
  with no pane, the host headings, and Clawd in octants, wearing the
  session's bandana, in the pose its state and subagents give it at its
  phase (running, squatting, waiting, waiting on subagents, standing)
- Its rows are byte-identical to `format`'s for the same tagged
  sessions, width, warn level and time, in every frame
- Each distinct pose is drawn once a render, as `format` does
- bash still gathers the tagged sessions each second; the binary
  renders when a gather lands

**Key interfaces:**
- The tagged-session lines `collect` writes, and the fzf item format the
  rows hold (tab fields, then display lines, NUL-terminated)
- `format`'s definitions: the poses, the octant table, `bandana` and
  `djb`, `ctxline`, `faded`, `animation`; the jq is the reference until
  this slice replaces it
- `CLAUDE_CTX_WARN` and the pane width

**Acceptance criteria:**
- [ ] Rows byte-identical to `format`'s over fixtures for every state:
      idle, working, waiting, idle and waiting with subagents, a session
      with no pane, a host unreachable, no context record, context at
      and past the warn level, two widths
- [ ] Every pose keeps each 2x4 cell to two colours, the background
      counting as one, by a count over the binary's poses
- [ ] The live loops' CPU measured before and after by the stage profile
- [ ] The Maintainer judges the sidebar unchanged by eye

**Out of scope:**
- New poses or animations
- Gathering the sessions (032)
