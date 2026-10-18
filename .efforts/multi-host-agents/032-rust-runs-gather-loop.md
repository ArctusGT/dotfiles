## The Rust binary gathers the sessions, reading local ones natively

**Category:** enhancement
**Summary:** The sidebar's gather loop moves into the Rust binary, which reads this host's sessions from their state files itself rather than spawning `claude-sessions`, and polls remote hosts over ssh as today
**Status:** ready-for-human
**blocked-by:** [multi-host-agents/031-rust-renders-frames]

**Current behavior:**
`gathering` collects every second, or at once on a kick: it spawns
`claude-sessions` and `tag` for this host, an ssh per remote host
running `claude-sessions` there, and tmux to find the frame's right
pane. In the live sidebar, before 029, `claude-sessions` cost 5.7% of a
core and the rest of the gather 3.2%.

**Desired behavior:**
- The binary gathers every second and at once on a kick through the
  kick fifo, as today, and draws (031) and serves (030) from what it
  gathers, with no bash loop left
- This host's sessions are read from the state directory, Claude Code's
  session records and the default tmux server's panes in-process, with
  the same result as `claude-sessions`: the same sessions, the same
  left-out ones, the same pane for each, and dead sessions' files
  removed
- Remote hosts are polled over ssh with the same options and master
  connection, running their own `claude-sessions`, in parallel, an
  unreachable one marked as today
- `pick N` finds entry N in `rows.0`, from the number and the host,
  pane, relay and session fields its rows carry, so the entries are
  numbered in one place; no tagged file is written any more
- `claude-sidebar` with no arguments starts fzf and the binary only;
  `pick`, `jump`, `edge`, `attach` and `kick` stay in bash

The Maintainer chose native reading over spawning `claude-sessions`
from the binary, and `pick` reading `rows.0` so the tagged file goes.

**Key interfaces:**
- `claude-sessions`' output and its rules for liveness, spares,
  background jobs and pane matching; it is the reference, and stays the
  remote hosts' source
- The kick and gathered fifos, `jumped`, `moving.*`, and `rows.0`,
  which `pick`, `position` and `step` read
- The ssh options and the remote command, including the relay pane
  after `\036`

**Acceptance criteria:**
- [x] The binary's local sessions equal `claude-sessions`' output over
      the same state, records and panes, by a script, including a dead
      session, a spare, a background job and a pane showing another
      session
- [x] A kick gathers at once; a jump during a gather does not take the
      cursor back, as today
- [x] `pick N` jumps to the entry numbered N on screen, for a local and a
      remote entry, with no tagged file in the cache; and with no
      sidebar running, as today
- [x] An unreachable host shows `unreachable` within one ssh
      ConnectTimeout, and the others keep updating
- [x] The live loops' CPU measured before and after by the stage
      profile, nothing moving and one working; the target is about 3 to
      4% of a core, measured against 18.2% before 029
- [ ] The Maintainer uses the sidebar for a session of work, jumping
      locally and to mgmt-1

**Out of scope:**
- Replacing `claude-sessions` on remote hosts
- `jump`, `edge`, `attach`, `kick`

**For the Maintainer:**
- From the structural review: a typed `Session` (and `State`) for the
  renderer, parsed from each session's JSON once, would remove
  `joined`, `last_part`'s `"null"`, `short_id`'s fallback and
  `Value::above_zero`. It changes the rows only for sessions
  `claude-session-state` never writes (a missing `cwd` would show blank
  rather than `null`). Left for a yes: the rows were kept byte-identical
  to the jq-era rows instead. `sessions` itself stays JSON, as it passes
  each state file through field for field.
- From the structural review: `attach` still runs the bash
  `claude-sessions` on the Mac, so two local readers of one set of rules
  now run side by side, both removing dead sessions' files. `attach`
  could read `claude-sidebar-loop sessions` instead (one word, its jq
  kept), leaving `claude-sessions` the remote hosts' source only. Not
  done: the slice keeps `attach` as it is.
- The comparison and profile scripts (`compare-sessions.sh`,
  `compare-rows.py`, `e2e-032.sh`, `profile-032.sh`) are scratchpad
  files, like 030's and 031's. Say if they should be shipped.
- In 26 end-to-end starts, once the cursor did not move to the entry on
  show when the sidebar started, under load right after a build; the
  same run's later checks, from the first jump on, passed. Not
  reproduced since; possibly the first gather, now far quicker, posting
  `pos` before fzf has loaded its first rows.
- mgmt-1 built and tested the crate offline (cargo 1.92) from a `git
  archive`, and its `sessions` matched that revision's `claude-sessions`
  byte for byte over a fixture there. mgmt-1 needs the binary after
  pulling, as `Readme.md`'s setup says. On mgmt-1, from `~/dotfiles`:

  ```bash
  cargo install --locked --path claude-sidebar-loop --root ~/.local
  ```
- The sessions' divergences from `claude-sessions` on files the hooks never
  write are in `sessions.rs`' module comment.
