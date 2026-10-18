## Clawd runs while his session works

**Category:** enhancement
**Summary:** A working session's Clawd plays a four-frame run at 8 fps in the sidebar, the bandana ribbon streaming behind him, and nothing redraws while no entry animates
**Status:** ready-for-closure
**blocked-by:** [multi-host-agents/024-bandana-per-session]

**Current behavior:**
The sidebar rebuilds its rows once a second (collect, then format) and posts
`reload-sync` to fzf only when they change. Clawd's pose is still.

**Desired behavior:**
- While a session is `working`, its Clawd runs: body shifted right, both
  arms swept left, slanted eyes, the four feet stepping in four distinct
  frames, the ribbon streaming left in a different shape each frame and
  cycling through the bandana's three stripe colours
- The frames play at 8 fps, from rows formatted out of the cached sessions;
  collecting from the hosts stays once a second
- With no entry animating, the sidebar redraws no more often than today
- Waiting and idle entries stay still (idle squats are 026)

The Maintainer chose 8 fps, and confirmed fzf neither flickers nor moves the
cursor at that rate, with `prototype-fzf-redraw.py` on the branch
`prototype/clawd-bandana` (c7a2438). Its run frames (`RUN_BODY`, `RIBBONS`,
`FEET`) are the starting point.

**Key interfaces:**
- `claude-sidebar`'s `serve` loop, `refresh`, `format` and the rows cache;
  `pick` and `position` read the rows, and must not see half of one
- The cost, measured: `format` about 18 ms a run on 11 entries
  (`format` timed 20 times on the cached sessions); `reload-sync` 0.6 ms
  median and fzf about 4% CPU at 8 fps (prototype, throwaway tmux server)

**Acceptance criteria:**
- [x] A working session's Clawd runs through four distinct frames at about
      8 fps; the others stay still
- [x] Every run frame keeps each cell to two colours at most, by the count
      script from 024
- [x] With no session working, `reload-sync` posts no more often than once
      a second, measured by counting them for a minute
- [x] The CPU the sidebar takes while one session works, measured and
      stated in the commit
- [x] The Maintainer sees no flicker and the cursor stays put, in the frame

**Out of scope:**
- Idle squats (026) and a waiting wave
- A frame rate per state, or one the Maintainer sets

## Comments

2026-10-18: for the Maintainer, from review, beyond what the slice asked.
Measurements are 52f9ff9's: a scratch copy of the script logging each
fzf post and its loops' CPU, 11 stubbed sessions, a throwaway tmux
server, 60 s.

- Render the frames at a gather, not at a tick. `serve` formats a whole
  frame every tick: about 19 ms of jq (`format` timed 40 times on the
  stubbed sessions), a `tmux display` fork for the width, and every still
  picture rebuilt, which is most of its 31% of a core while one session
  works. `gather` could write the four frames once a second, as `rows.0`
  to `rows.3` from one jq run, and a tick would then only post
  `reload-sync` on the next one. `serve` would lose most of its flags,
  and "something animates" would become frame 0 differing from frame 1,
  so `picture` would be the only place deciding what moves. The brief
  asked for a format per tick with a frame counter; 026's squats, a
  15-frame cycle per session in the prototype, would weigh on how many
  frames a gather renders.

---

2026-10-18: also from review.

- With nothing working the sidebar takes 12.6% of a core against 11.3%
  before 025, on the same fixture. Not attributed; `serve` now runs a jq
  per gather to see whether anything works.
- `kick` still reaches `gathering` through a flag file polled 5 times a
  second, as it did before this slice: a sleep fork each, and up to 200 ms
  before a jump shows; up to 325 ms while a session works, since a gather
  then waits for the next tick. A second fifo, opened read-write by
  `kick` so a stale one cannot hang a tmux hook, would end the polling.

---

2026-10-18: after the Maintainer's look in the frame (cursor lag, and a
jump-back after Cmd+Up/Down), 8f75ee4 took the first note above and the
kick fifo from the second: frames are drawn once a gather, and a gather a
kick overtook no longer moves the cursor. Numbers and the race probe are
in its message; the idle sidebar is back to 11.5% of a core against 11.3%
before 025.

For 026: the frames are one cycle shared by every entry, `frame_count`
files a gather, with a session's pose picked from the frame number. Idle
squats, a 15-frame cycle per session with its own phase in the prototype,
would need `frame_count` raised to the longest cycle (or a common multiple
of the cycles) and a per-session offset into it, so a gather would format
that many frames: on 11 stubbed sessions one jq run takes 20 ms for 1
frame, 46 ms for 4 and 131 ms for 15 (`format` timed 20 times each), so 15
frames would cost about 13% of a core at one gather a second. "Something
moves" as rows.0 against rows.1 would also miss a squat that starts later
in the cycle; comparing rows.0 with each other frame, or the files' count
of distinct sums, would cover it.

---

2026-10-18: the Maintainer's mgmt-1 entries still bounced (fzf's cursor
9, 7, 9, 7 within 2 s). A jump now marks itself moving before it moves
the right pane's client, and once the move has landed, after the ssh for
a host, writes a new value to `$cache/jumped`; `serve` moves the cursor
only for a gather that began under the value still standing, with no jump
moving. The probe now stands in a fake host behind a stub ssh with a set
round trip; numbers in the commit. The Maintainer asked for the jump to
be counted before it moves; counted only then, a gather that starts after the count but
reads the host before its ssh lands still matches, so the count also
moves once the move is done, and the mark covers the time between.

With the hooks tmux/.tmux.conf.local sets, which kick the sidebar at
every host jump, the probe got no stale post out of the script before
this change; with them left out, 28 in 18 bursts. So the bounce seen on
mgmt-1 may have had a cause the probe does not model. One candidate:
Cmd+Up/Down in a host window runs `edge`, which asks the host for its
active pane over ssh; while the previous jump's ssh is still in flight
that is the old pane, so `step` picks from the old entry. If the recheck
still bounces, fzf's cursor polled as before beside the sidebar's posts
would tell the two apart.

From review, left open: two host jumps whose ssh calls land out of order
leave the host on the earlier pick, the same 9, 7, 9, 7 to the eye; a
jump killed between its mark and its move leaves `moving.<pid>`, which
stops the cursor following until the sidebar restarts; and a pick does
not tell `serve` where it put the cursor, so switching back by hand to
the session `serve` last posted does not move it.
