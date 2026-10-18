## Clawd squats while his session is idle and its subagents work

**Category:** enhancement
**Summary:** An idle session with subagents running (`+N`) shows its Clawd squatting, standing to down and back, as its wait-for-subagents animation; an idle session without subagents stands still
**Status:** ready-for-closure
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
- [x] The Maintainer judges the squat by eye

**Out of scope:**
- The waiting animations (arms, stamping, blinking, eyes looking around),
  prototyped before they get a slice
- Moving the renderer to Rust, considered once the look settles

## Comments

For the Maintainer, from implementation and review. Loop CPU is a
scratch copy of the script logging each fzf post and its loops' bash
`times` (children included), 11 stubbed sessions, a throwaway tmux
server, 60 s; clashes are counted by evaluating the poses from the
script's jq and counting 2x4 cells over two colours, the empty
background counting as one.

- The squat redrawn so the arms go down with him (9068f8a): down drops
  the head a whole cell row, eyes on row 8, arms 9-10, feet 11, the
  ribbon floating above; the crouch keeps the standing head (no smaller
  drop fits) and lowers the arms a row to 7-8, its ribbon drop a row
  longer to fill the cell beside them. 0 clashes in every pose.
- Each entry is formatted once a gather and its distinct poses drawn
  once (b0beb4a), and a tick whose frame repeats the one fzf holds
  posts nothing (3273231). Rows byte-identical to 9068f8a's on four
  fixtures. Against 228d432:
  - nothing moving: loops 10.1% of a core against 13.3% (10.3% before
    026)
  - one working: 17.5% against 19.3%, 469 reloads a minute
  - one squatting: 7.6% against 22.5%, 192 reloads a minute against 471
  - `format`, 12 frames on 12 entries, the Mac loaded: 25 ms against
    95 ms, and 43 ms for 4 frames before 026
- A gather while something moves still makes the next tick post, so
  about one repeated frame a second reaches fzf for new rows.
- "the eyes narrow to slits" traces to the prototype, where the standing
  eyes were two rows and the squat's one. 027 made the standing eyes
  one-row slits, so the squat's match them and only move down.
