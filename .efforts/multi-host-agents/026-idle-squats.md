## Clawd does the odd squat while his session is idle

**Category:** enhancement
**Summary:** An idle session's Clawd now and then squats, standing to down and back, at moments of his own, using 025's animation loop only while a squat plays
**Status:** ready-for-agent
**blocked-by:** [multi-host-agents/027-bandana-one-row-lower]

**Current behavior:**
After 025, an idle Clawd stands still.

**Desired behavior:**
- An idle Clawd squats every so often: the bandana drops, the arms come up
  under it, the eyes narrow to slits below them, then he stands again
- Each session squats at its own moments, so the sidebar's idle entries
  never squat in step
- Between squats nothing redraws for them

The Maintainer drew the squat in two poses. Drawn so no cell holds three
colours, the squat is two frames, stand and down (`SQUAT_DOWN` in
`prototype-clawd-run.py`, branch `prototype/clawd-bandana`, c7a2438): the
halfway pose puts the bandana across a cell boundary over the eyes. A third
frame is welcome if one can be drawn within that rule. The squat is to be
redrawn to 027's lower bandana (head-top, bandana, eyes on whole blocks),
which SQUAT_DOWN predates.

**Key interfaces:**
- 025's animation loop, and whatever tells it an entry is animating
- A session's squat moments from its id and the clock, so nothing is stored

**Acceptance criteria:**
- [ ] Idle entries squat now and then, not together
- [ ] Every squat frame keeps each cell to two colours at most, by the
      count script from 024
- [ ] Between squats, with no session working, `reload-sync` posts no more
      often than once a second
- [ ] The Maintainer judges how often and how long by eye

**Out of scope:**
- A waiting wave, which the Maintainer welcomes once these land
