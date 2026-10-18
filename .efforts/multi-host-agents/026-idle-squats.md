## Clawd squats while his session is idle and its subagents work

**Category:** enhancement
**Summary:** An idle session with subagents running (`+N`) shows its Clawd squatting, standing to down and back, as its wait-for-subagents animation; an idle session without subagents stands still
**Status:** ready-for-agent
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
- [ ] An idle session with subagents running squats; once they finish it
      stands still within about a second
- [ ] Working and idle-without-subagents entries behave as before
- [ ] Every squat frame keeps each cell to two colours at most, by a
      count script
- [ ] With nothing working or squatting, `reload-sync` posts no more often
      than once a second
- [ ] The Maintainer judges the squat by eye

**Out of scope:**
- The waiting animations (arms, stamping, blinking, eyes looking around),
  prototyped before they get a slice
- Moving the renderer to Rust, considered once the look settles
