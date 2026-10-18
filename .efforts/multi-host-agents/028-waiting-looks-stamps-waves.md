## Clawd looks about, stamps and waves while his session waits

**Category:** enhancement
**Summary:** A waiting session with no subagents running shows its Clawd looking left and right, stamping, then waving, once every 6 seconds; between them he stands still
**Status:** ready-for-agent
**blocked-by:**

**Current behavior:**
A waiting Clawd stands still in yellow. A working one runs (025) and an
idle one with subagents running squats (026); everything else stands.

**Desired behavior:**
- While a session is `waiting` and has no subagents running, its Clawd
  plays one loop of 48 frames, 6 s at 8 fps, in this order: the eye slits
  a pixel left, centre, a pixel right, centre; both pairs of feet stamp in
  turn; the left arm rises beside the bandana and waves; then he stands
  for the rest of the loop
- Each waiting entry plays at its own phase, from its session id as the
  squat does, so waiting entries do not move in step
- Working, squatting and still entries behave as before; a waiting
  session with subagents running stands, as today

The Maintainer chose these three motions, and running them one after
another over running them at once, by eye from the prototype
`prototype-clawd-wait.py` on branch `prototype/clawd-bandana` (8c834c3).
Its `in turn` animation is the reference: `SEQ_EYES`, `SEQ_FEET` and
`SEQ_ARM` give the pose for each of the 48 frames, and `pose()` builds it
from today's standing Clawd. The eyes, the feet and the left arm each sit
in cells that hold only body and background, so no pose clashes; the
right arm cannot rise past eye level, since the ribbon tail shares that
cell.

**Key interfaces:**
- The sidebar's `animation($r)`: which poses a session cycles through and
  its phase; a third case for `waiting`
- `frame_count`, the frames a gather renders: 12 today, a multiple of
  every animation's length, so 48 covers run (4), squat (12) and this
- 026's costs: each distinct pose drawn once a gather, and a tick whose
  frame repeats the one fzf holds posts nothing. A 48-frame loop is
  mostly the standing pose, but a gather still formats every entry for
  48 frames, against 12 today

**Acceptance criteria:**
- [ ] A waiting session with no subagents looks about, stamps and waves
      in that order, once a loop; when it stops waiting its Clawd takes
      the new state's pose within about a second
- [ ] Working, squatting, idle and waiting-with-subagents entries behave
      as before
- [ ] Every waiting pose keeps each cell to two colours at most, by a
      count script
- [ ] With nothing working, squatting or waiting, `reload-sync` posts no
      more often than once a second
- [ ] Loop CPU with nothing moving, one working and one waiting, and
      `format`'s time for a gather, measured against main before the
      slice by 026's method; a rise in the nothing-moving case is named
      to the Maintainer before closure
- [ ] The Maintainer judges the waiting Clawd by eye

**Out of scope:**
- Moving the renderer to Rust
- Marking a session `working` from tool-use hooks
- Blinking, foot tapping, raising both arms: prototyped, not chosen
