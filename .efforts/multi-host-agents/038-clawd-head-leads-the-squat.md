## Clawd's bandana moves with his eyes as he runs, and his head leads the squat

**Category:** enhancement
**Summary:** Running, the bandana's colours shift sideways in step with the eyes' glance, so the head reads as moving; squatting, the head goes down first and the arms follow as the body and legs compress, he holds low, the head comes up first and the arms follow as he decompresses, and he holds at the top

**Status:** ready-for-agent
**blocked-by:**

**Current behavior:**
Running is four frames at eight a second: the ribbon streams left in a
cell-a-frame colour cycle, the feet step, and the eyes glance a pixel
right on the second and fourth frames. The bandana and its stripes hold
still, so the glance reads as the eyes alone moving.

A squat is four frames, a crouch, two squats and a crouch, half a
second in all: the head drops a whole cell row with the arms and the
body in the same frame, and comes back the same way. Idle with
subagents running, he stands eight frames and squats; waiting with
subagents running, a squat sits in each of the wait's three longest
rests, of four, six and nine frames. Every animation's length divides
the 48 frames a gather renders.

Every pose keeps each octant cell, two pixels by four, to two colours
with the empty background counting as one, since a terminal cell draws
a foreground and a background and no more; `octant` draws a cell as one
colour and empty, or as two colours and full. The cells start at pixel
rows 0, 4 and 8, so the head-top and the bandana share one cell row,
and the eyes, arms and body the next.

**Desired behavior:**
- Running, the bandana's colours move sideways with the eyes on each
  glance, by the smallest step the cell rule allows, and back; the
  ribbon's cycle, the feet and the eyes are as today
- The squat goes: head down first, the eyes and bandana with it, the
  arms still up; the arms follow down as the body and legs compress to
  the squat; a brief hold low; the head up first, the arms still low;
  the arms follow up as the body and legs decompress to standing; a
  brief hold at the top. A hold is about a quarter of a second, and the
  whole squat about a second, with each phase at least one frame; the
  Maintainer judges the timing by eye
- The squat keeps its place in the idle-with-subagents animation and in
  the wait's rests, which lengthen if the squat outgrows them, the
  wait's other steps unchanged; every animation still divides the 48
  frames
- Every pose still keeps each cell to two colours, by the existing test

**Key interfaces:**
- The pose constants and `run`, `squat_down`, `Wait` and the animation
  tables in the renderer, and `Clawd::new`, which indexes them
- The two existing tests, every pose two colours a cell and every
  animation dividing `FRAME_COUNT`
- The stage profile check, as more distinct poses mean more frames
  rendered per gather

**Acceptance criteria:**
- [ ] The run's four frames show the bandana's colours stepped sideways
      on the glance frames and back on the others, by a test over the
      poses' bandana rows
- [ ] The squat's frames, in order, show the head a cell row lower
      before the arms move, the arms lower before the body compresses,
      a hold low, the head back up before the arms, the arms up before
      the body decompresses, and a hold at the top, by a test over the
      poses' rows
- [ ] Both existing pose tests pass unchanged
- [ ] The live loops' CPU measured by the stage profile, 60 s, one
      working and one idle with subagents, within a point of a core of
      before
- [ ] The Maintainer watches a running and a squatting Clawd and judges
      the head's motion and the squat's timing right

**Out of scope:**
- New poses for waiting or standing
- The ribbon's cycle, the feet and the eyes in the run
- A fourth colour in a cell, which a terminal cannot draw

**For the Maintainer:**
- A one-pixel sideways move of the stripes cannot be drawn. The stripes
  are two pixels wide on cell boundaries, under two rows of head-top in
  the same cell; shifted a pixel, a cell holds two stripe colours and
  the head-top's, three. The smallest step the rule allows is a whole
  cell, two pixels, which is what the brief asks for. A one-pixel move
  becomes possible only if the head-top rows above the bandana go, so
  the bandana is the top of the head and its cells hold stripe colours
  alone; that is a change to how Clawd looks standing too. The
  Maintainer accepted the whole-cell step.
- A head drop short of a cell row meets the same rule, as the crouch's
  comment records: the bandana lands in the eyes' cell with the body
  and the empty eye. So the head leads by a whole cell row, which the
  squat pose already has, and the phases are ordered by which parts
  have moved rather than by pixel easing.
