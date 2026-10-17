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
- [ ] A working session's Clawd runs through four distinct frames at about
      8 fps; the others stay still
- [ ] Every run frame keeps each cell to two colours at most, by the count
      script from 024
- [ ] With no session working, `reload-sync` posts no more often than once
      a second, measured by counting them for a minute
- [ ] The CPU the sidebar takes while one session works, measured and
      stated in the commit
- [ ] The Maintainer sees no flicker and the cursor stays put, in the frame

**Out of scope:**
- Idle squats (026) and a waiting wave
- A frame rate per state, or one the Maintainer sets
