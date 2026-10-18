## The Rust binary renders the sidebar's frames

**Category:** enhancement
**Summary:** The sidebar's frames are rendered inside the Rust binary from the gathered sessions, in place of `format`'s jq and `draw`'s perl split, byte-identical to today's rows
**Status:** ready-for-human
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
- [x] Rows byte-identical to `format`'s over fixtures for every state:
      idle, working, waiting, idle and waiting with subagents, a session
      with no pane, a host unreachable, no context record, context at
      and past the warn level, two widths
- [x] Every pose keeps each 2x4 cell to two colours, the background
      counting as one, by a count over the binary's poses
- [x] The live loops' CPU measured before and after by the stage profile
- [ ] The Maintainer judges the sidebar unchanged by eye

**Out of scope:**
- New poses or animations
- Gathering the sessions (032)

**For the Maintainer:**
- From the structural review, before 032: an entry's number is now
  counted in Rust (`frames::render`) and again in `pick`'s jq over
  `tagged`, two copies of one rule in two languages; if they disagree,
  `pick N` jumps to another entry than the one numbered N. `pick` could
  read fields 1-5 of `rows.0`, as `step` and `position` already do,
  leaving the rows the one source. That would also dissolve 032's
  criterion that `tagged` is still written whole for `pick`.
- From the structural review, for 032: the renderer reads the raw JSON
  values in jq's loose terms (`joined`, `last_part`'s `"null"`,
  `above_zero` over every type), as byte-identity was measured against
  jq. Once 032 builds sessions natively, a typed `Session` and `State`
  parsed once at the boundary removes most of that.
- From the spec review, for 032: the rows print `used_pct` and
  `subagents` as `tagged` writes them, which matches `format` because
  `tag`'s jq writes them in jq's number format. Sessions gathered
  without jq must write numbers the same way, or a `1e2` shows as is.
- From the spec review: a gather whose `tagged` cannot be rendered is
  logged and skipped, where `draw` failing still announced it, so the
  cursor does not follow that one gather. Left as is, as `collect`
  writes `tagged` with jq.
- The second criterion's "each distinct pose is drawn once a render"
  is, as in `format`'s `pictures`, once per session a render: each
  session has its own bandana.
- The byte-identity harness (`compare-031.py`) and the profile
  (`profile-031.sh`) are scratchpad scripts, so the numbers in 3783cac
  and be9260a cannot be re-taken from the repo. Say if they should be
  shipped.
- The Mac has the binary installed from this slice's last code commit;
  the live sidebar picks it up on its next restart. The new
  `claude-sidebar` needs it: with 030's binary, `claude-sidebar-loop
  rows` is taken for a cache directory and the sidebar starts empty.
  mgmt-1 needs it after pulling (`Readme.md`, from `~/dotfiles`):
  `cargo install --locked --path claude-sidebar-loop --root ~/.local`.
