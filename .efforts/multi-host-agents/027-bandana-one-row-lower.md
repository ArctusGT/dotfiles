## The bandana one row lower, standing and running

**Category:** enhancement
**Summary:** The sidebar's standing and running Clawd wear the bandana below a strip of head, with the eyes just under it, as the Maintainer chose from the "low" prototype
**Status:** ready-for-human
**blocked-by:** []

**Current behavior:**
The bandana replaces Clawd's head-top: rows 2-3 of the 16x12 frame, a
ribbon tail above it on rows 0-1, the eyes on rows 4-5. Running, the body
sits at columns 3-15 and the ribbon streams through the one clear cell on
the left, in one colour a frame.

**Desired behavior:**
- Standing: every feature two rows on whole 2x4 blocks: head-top (rows
  0-1), bandana (2-3), eyes with a two-wide ribbon drop beside them (4-5),
  arms (6-7), body (8-9), feet (10-11); a one-colour ribbon tail at the
  bandana's end
- Running: the body two columns right (4-15), the stripes at its far end,
  slit eyes, both arms swept left, the four foot steps as now; the ribbon
  streams through the two clear cells on the left, one stripe colour each,
  changing each frame
- Every frame keeps each 2x4 cell to two colours at most, the background
  counting as one
- A cell lit in two colours is drawn as the prototype draws it: one as the
  glyph's colour, the other as the cell's background colour
- Nothing else changes: the colours a session wears, the animation loop,
  the entry's four lines

The frames are `LOW_STAND`, `LOW_RUN`, `RIBBON_L`, `RIBBON_R`,
`RIBBON_COLOURS` and `LOW_FEET` in `prototype-clawd-low.py` on the branch
`prototype/clawd-bandana` (bc2e9ee): 0 clashes in all five. The Maintainer's
own mock trimmed to 12 rows is the same standing frame pixel for pixel.

**Key interfaces:**
- `claude-sidebar`'s jq frame defs (`stand`, `run` and its parts),
  `octant` and `picture`

**Acceptance criteria:**
- [x] The standing and the four run frames match the prototype's, row for
      row
- [x] Each has no cell holding more than two colours, by a count script
- [ ] The Maintainer judges it by eye in the frame, standing and running

**Out of scope:**
- The squat (026), redrawn to this head on its own slice
- Any change to the animation loop or to the colours a session wears

## Comments

**How `octant` draws a cell.** The prototype's renderer draws a cell lit
in two colours with a foreground and a background colour, and these
frames need that: the head-top and the bandana share each cell of the top
row. The Maintainer chose to teach `octant` the same. A cell lit in two
colours draws its first lit pixel's colour as the glyph and the other as
the cell's background, then resets the background at the end of the cell.
A cell in one colour draws as before.

What remains for the Maintainer to see by eye, beside criterion 3:

- **The cursor line.** fzf draws its highlight (`bg+`, the background of
  the line under the cursor) again after each cell, so it carries on for
  the ribbon tail and the text. The bandana cells are full, so they keep
  their own colours, and fzf keeps its bold across them. Method: fzf
  0.74.5 with the sidebar's flags, on rows formatted from a live collect,
  in a `tmux -L` server, captured with `capture-pane -e` with the cursor
  on a working entry and then on a dimmed one.
- **Dimmed entries.** A background entry is dimmed with SGR 2 (Select
  Graphic Rendition 2, faint), which Ghostty applies to the text colour
  only (`faint-opacity`, "the opacity level of the faint text", 0.5). A
  dimmed cell with a background therefore turns faint off and draws both
  colours halfway to the terminal background, the shade faint text takes.
  `faded` holds Ghostty's TokyoNight Night shades for the palette colours
  and its background, #1a1b26.

**Open for the Maintainer**: the dim could instead fade the whole palette
in `picture` and drop SGR 2 from the picture. That keeps it in one place
rather than two, but one-colour cells would then follow `faded`'s table
rather than the terminal's own faint.
