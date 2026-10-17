## Clawd runs while his session works

**Category:** enhancement
**Summary:** A working session's Clawd plays a four-frame run at 8 fps in the sidebar, the bandana ribbon streaming behind him, and nothing redraws while no entry animates
**Status:** ready-for-agent
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
- [ ] The Maintainer sees no flicker and the cursor stays put, in the frame

**Out of scope:**
- Idle squats (026) and a waiting wave
- A frame rate per state, or one the Maintainer sets

## Comments

For the Maintainer, from review, beyond what the slice asked:
- Render the frames at a gather, not at a tick. `serve` formats a whole
  frame every tick: about 19 ms of jq, a `tmux display` fork for the
  width, and every still picture rebuilt, which is most of its 31% of a
  core while one session works. `gather` could write the four frames once
  a second, as `rows.0` to `rows.3` from one jq run, and a tick would then
  only post `reload-sync` on the next one. `serve` would lose most of its
  flags, and "something animates" would become frame 0 differing from
  frame 1, so `picture` would be the only place deciding what moves. The
  brief asked for a format per tick with a frame counter; 026's squats, a
  15-frame cycle per session in the prototype, would weigh on how many
  frames a gather renders.
- `kick` still reaches `gathering` through a flag file polled 5 times a
  second, which is most of what the idle sidebar now spends over the old
  one (12.6% of a core against 11.3%). A second fifo, opened read-write by
  `kick` so a stale one cannot hang a tmux hook, would end the polling.
