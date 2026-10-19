## Sidebar loop follow-ups: one local reader, a typed session, owned caches, shipped checks

**Category:** enhancement
**Summary:** Four follow-ups from 030 to 032 the Maintainer accepted: `attach` reads sessions through the binary, the renderer parses each session into a typed `Session`, a sidebar removes only the cache it created, and the comparison and end-to-end checks ship in the repo
**Status:** ready-for-closure
**blocked-by:** [multi-host-agents/032-rust-runs-gather-loop]

**Current behavior:**
- `claude-sidebar attach` runs the bash `claude-sessions` on the Mac, so
  two readers of one set of rules run side by side there, both removing
  dead sessions' files
- The renderer reads each session as loose JSON, with fallbacks for
  fields `claude-session-state` always writes
- Every sidebar on a host uses one fixed cache directory and removes it
  whole on exit; a second sidebar, or a test run on the real TMPDIR,
  deletes a running sidebar's cache and freezes it (seen live during
  032, and a possible cause of two respawns that lost the pane)
- The checks that proved 029 to 032 (session and row comparison, end to
  end, CPU profile) live only in one session's scratchpad, so their
  numbers cannot be taken again from the repo

**Desired behavior:**
- `attach` takes its session from `claude-sidebar-loop sessions`; the
  bash `claude-sessions` is then the remote hosts' source only, and
  says so
- The renderer parses each session once into a typed `Session` (and
  `State`), dropping the fallbacks only malformed input reached; rows
  stay byte-identical for every session the hooks write
- A sidebar removes its cache on exit only while that cache is still its
  own; a sidebar starting while another's cache exists takes it over
  without the old one's exit removing the new one's files
- The comparison and end-to-end checks run from the repo, against
  throwaway tmux servers and their own TMPDIR, never the live cache; the
  CPU profile ships too if it can without machinery beyond a script

The Maintainer accepted all four from 032's review notes.

**Key interfaces:**
- `claude-sidebar-loop sessions` and `attach`'s use of the session row
- The renderer's session input and its rows
- The cache directory, its exit trap, and what `pick`, `kick`, `jump`
  and `edge` expect to find there
- Where checks live in the repo, and how each says what it does
  (`--help`, `.agents/rules/scripts-shape.md`)

**Acceptance criteria:**
- [x] `attach` opens or finds a session's pane as today, with no
      `claude-sessions` spawn on the Mac
- [x] Rows byte-identical to 032's over the shipped row comparison's
      fixtures
- [x] Respawning a sidebar with `respawn-pane -k` in a throwaway tmux,
      ten times, leaves a working sidebar every time; a second sidebar
      started and stopped beside a running one leaves the running one
      updating
- [x] The shipped checks run from a clean checkout on the Mac and pass;
      none touches the live cache, by running them while a sidebar runs
      and seeing it keep updating
- [x] The Maintainer restarts the live sidebar and uses it

**Out of scope:**
- Replacing `claude-sessions` on remote hosts
- New sidebar behaviour

**For the Maintainer:**
- A session with no `cwd` in its state file now shows a blank repo where it
  showed `null`; `claude-session-state` writes `cwd: null` only when neither
  the hook input nor the old state has one. The row fixtures all carry a cwd.
- A session missing `since`, `subagents` or a valid `state` is now left out
  of the list with a line in `claude-sidebar.log` each gather, where it was
  drawn with defaults. A remote host's `claude-session-state` older than the
  fields would lose its sessions from the sidebar. Not measured.
- `claude-sidebar`'s link to the running sidebar's cache is taken once its
  fzf listens and made again within 2 s if another sidebar's exit takes it,
  so for up to about 2 s after a restart `pick`, `kick` and `jump` find no
  cache. With two sidebars alive the link names the last to start, and a
  `jump` from the other's fzf acts on that one's cache. The first start
  after this commit, against a running sidebar that still keeps a real
  directory at the link, takes the link only once that sidebar's exit has
  removed the directory.
- A sidebar that dies without its exit trap (SIGKILL) leaves its
  `claude-sidebar.PID` directory behind; nothing sweeps them.
- `attach` now needs `claude-sidebar-loop` on every host a jump reaches
  over ssh. mgmt-1 has it.
- The comparison's first-pass respawn case did not fail unloaded on the old
  script; only with the old exit trap made 0.5 s slower, as under load.
