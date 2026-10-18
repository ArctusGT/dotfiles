## Clawd squats while his session is idle and its subagents work

**Category:** enhancement
**Summary:** An idle session with subagents running (`+N`) shows its Clawd squatting, standing to down and back, as its wait-for-subagents animation; an idle session without subagents stands still
**Status:** ready-for-human
**blocked-by:** [multi-host-agents/027-bandana-one-row-lower]

**Current behavior:**
After 025, an idle Clawd stands still whether or not its subagents work;
only the `+N` count shows them.

**Desired behavior:**
- While a session is `idle` and has running subagents, its Clawd squats
  over and over: the bandana drops, the arms come up under it, the eyes
  narrow to slits below them, then he stands again
- A working session runs (025), whatever its subagents; a waiting one
  keeps its own animation (a later slice); an idle one with no subagents
  stands still
- Squatting entries keep their own phase, so they do not squat in step
- With nothing working or squatting, nothing redraws more than today

The Maintainer chose squats as the subagent-wait animation, in place of
random squats, and only while the session itself is idle. The Maintainer
drew the squat in two poses (`SQUAT_DOWN` in `prototype-clawd-run.py`,
branch `prototype/clawd-bandana`, c7a2438), which predate 027's lower
bandana: the squat is redrawn to 027's head (head-top, bandana, eyes on
whole blocks), two frames or more, each cell two colours at most with the
background counting as one, which 027's renderer draws.

**Key interfaces:**
- 025's animation loop: `picture` decides what moves, a gather renders
  `frame_count` frames; a squat cycle longer than four frames raises it
  (025's closing notes: about 131 ms of jq for 15 frames per gather)
- The subagent count `claude-sessions` already carries
- A session's squat phase from its id, so nothing is stored

**Acceptance criteria:**
- [x] An idle session with subagents running squats; once they finish it
      stands still within about a second
- [x] Working and idle-without-subagents entries behave as before
- [x] Every squat frame keeps each cell to two colours at most, by a
      count script
- [x] With nothing working or squatting, `reload-sync` posts no more often
      than once a second
- [ ] The Maintainer judges the squat by eye

**Out of scope:**
- The waiting animations (arms, stamping, blinking, eyes looking around),
  prototyped before they get a slice
- Moving the renderer to Rust, considered once the look settles

## Comments

For the Maintainer, from implementation and review. Numbers are 228d432's
(scratch copy logging fzf posts and its loops' bash `times`, 11 stubbed
sessions, throwaway tmux server, 60 s).

- The cost of 12 frames a gather, paid every second even with nothing
  moving: loops 13.3% of a core against 10.3% before, `format` 68 ms
  against 27 ms. About 90% of `format` is `picture`, run per entry per
  frame although most entries draw the same picture in every frame
  (12 frames with `picture` stubbed: 11 ms). Two ways back, not taken:
  - render each entry's distinct pictures once and reuse them across
    the frames, a change to `format`'s main body; a still sidebar
    would then cost about one frame's worth, below today's
  - or a shorter cycle: 8 frames (stand 4, crouch, down 2, crouch)
    measured 48 ms, a squat every second rather than every 1.5 s
- `serve` posts `reload-sync` on every tick while anything moves, so one
  squatting session reloads fzf 8 times a second though 8 of its 12
  frames are the same standing pose (471 posts a minute). `draw` already
  compares frames, and could mark which ones repeat the last for `serve`
  to skip.
- "the eyes narrow to slits" traces to the prototype, where the standing
  eyes were two rows and the squat's one. 027 made the standing eyes
  one-row slits, so the squat's eyes match them and only move down.
- The crouch (feet one row shorter) is the Agent's halfway pose: a
  halfway with the head one row down puts the empty row, head-top and
  bandana in one cell row.
