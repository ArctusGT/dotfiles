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
- [x] A waiting session with no subagents looks about, stamps and waves
      in that order, once a loop; when it stops waiting its Clawd takes
      the new state's pose within about a second
- [x] Working, squatting, idle and waiting-with-subagents entries behave
      as before
- [x] Every waiting pose keeps each cell to two colours at most, by a
      count script
- [x] With nothing working, squatting or waiting, `reload-sync` posts no
      more often than once a second
- [x] Loop CPU with nothing moving, one working and one waiting, and
      `format`'s time for a gather, measured against main before the
      slice by 026's method; a rise in the nothing-moving case is named
      to the Maintainer before closure
- [ ] The Maintainer judges the waiting Clawd by eye

**Out of scope:**
- Moving the renderer to Rust
- Marking a session `working` from tool-use hooks
- Blinking, foot tapping, raising both arms: prototyped, not chosen

## Comments

For the Maintainer, from implementation. Loop CPU is a scratch copy of
the script writing each loop's bash `times` (children included) on every
pass and counting each fzf post, 11 stubbed local sessions (one in the
fixture's state, the rest idle), a throwaway tmux server, 60 s after a
5 s warm-up; `format` is the median of 9 runs on the gathered sessions
with the sidebar stopped. Main is 207dbb4's parent.

- Nothing moving, main and this slice alternated three times each: loops
  7.9, 6.6, 6.8% against 7.9, 7.1, 7.4% of a core, a rise of about 0.4
  points; 1 post a minute in both; `format` about 58 ms against 63 ms.
  Run to run, the same main measured anywhere from 6.1 to 8.2%.
- One working: 11.3% against 12.0%, 477 against 478 posts a minute,
  `format` 59 against 66 ms.
- One waiting (standing on main): 8.2% against 7.7%, 1 against 213 posts
  a minute (a working one posts 478), `format` 62 against 84 ms.
- 48 frames cost `format` almost nothing over 12: each distinct pose is
  drawn once and the frames repeat it, so 12 and 48 frames time within a
  few ms of each other in every fixture. The waiting fixture's extra
  22 ms is its 8 distinct poses.
