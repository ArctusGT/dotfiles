## The bandana one row lower, standing and running

**Category:** enhancement
**Summary:** The sidebar's standing and running Clawd wear the bandana below a strip of head, with the eyes just under it, as the Maintainer chose from the "low" prototype
**Status:** ready-for-agent
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
- Nothing else changes: colours, the animation loop, the entry's four lines

The frames are `LOW_STAND`, `LOW_RUN`, `RIBBON_L`, `RIBBON_R`,
`RIBBON_COLOURS` and `LOW_FEET` in `prototype-clawd-low.py` on the branch
`prototype/clawd-bandana` (bc2e9ee): 0 clashes in all five. The Maintainer's
own mock trimmed to 12 rows is the same standing frame pixel for pixel.

**Key interfaces:**
- `claude-sidebar`'s jq frame defs (`stand`, `run` and its parts) and
  `picture`

**Acceptance criteria:**
- [ ] The standing and the four run frames match the prototype's, row for
      row
- [ ] Each has no cell holding more than two colours, by a count script
- [ ] The Maintainer judges it by eye in the frame, standing and running

**Out of scope:**
- The squat (026), redrawn to this head on its own slice
- Any change to the animation loop or the colours
