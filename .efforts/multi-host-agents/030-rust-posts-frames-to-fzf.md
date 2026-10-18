## A Rust binary posts the sidebar's frames to fzf

**Category:** enhancement
**Summary:** The sidebar's `serve` loop moves into a long-lived Rust binary that posts each frame to fzf's socket itself, removing a `curl` spawn per frame, and the repo gains a Rust crate with a build step on each host
**Status:** ready-for-human
**blocked-by:**

**Current behavior:**
`serve` posts to fzf's `--listen` unix socket with `curl` for every
reload and cursor move: 8 a second while a Clawd moves. On the Mac a
post costs about 7.5 ms of CPU; one working session made 434 posts a
minute, 5.4% of a core of the loops' 18.2% in the live sidebar. bash
cannot reach a unix socket without spawning.

**Desired behavior:**
- A Rust binary does what `serve` does today, with the same timing and
  the same choices: it waits for each gather's line on the gathered
  fifo, reads `rows.N` and `repeats`, ticks 8 a second while a frame
  differs from another, posts `reload-sync` only when what fzf shows
  changes, and moves the cursor to the frame's right pane's entry under
  the same conditions (the `jumped` value, no `moving.*` mark)
- It speaks HTTP to fzf's unix socket from the process, keeping no
  `curl`, `perl` or `cksum` spawns
- bash still gathers and draws the rows; `claude-sidebar` with no
  arguments starts the binary in place of its `serve` loop
- The crate lives in the repo outside the stow packages; one command
  builds it and installs the binary where the sidebar finds it, on the
  Mac (cargo 1.97) and on Linux (EL10 ships cargo 1.92), and the
  README's prerequisites and setup say so

The Maintainer chose to move the sidebar to Rust as one long-lived
process, in three steps (this, 031, 032), after its live profile showed
most of its CPU in process spawns.

**Key interfaces:**
- The files and fifos `gathering` and `draw` leave in the sidebar's
  cache: `rows.N`, `repeats`, `gathered`, `jumped`, `moving.*`, and
  fzf's `fzf.sock`
- fzf's `--listen` HTTP API: `POST /` with an action in the body
- Where the binary is installed, and how the sidebar finds it

**Acceptance criteria:**
- [x] The build command builds and installs the binary on the Mac and on
      mgmt-1 from a clean checkout
- [x] With nothing moving, `reload-sync` posts no more often than once a
      second; with a Clawd moving, at 8 a second with no tick skipped
      for a slow post, as today
- [x] The cursor follows a jump in the frame as today, and does not jump
      back to an entry being browsed to
- [x] The live loops' CPU measured before and after by the stage
      profile, 60 s, nothing moving and one working; the post stage
      gone
- [ ] The Maintainer watches the sidebar animate and follow jumps

**Out of scope:**
- Rendering the frames (031) and gathering the sessions (032)
- `pick`, `jump`, `edge` and `attach`, which stay in bash

**For the Maintainer:**
- From the structural review, for 031: `Serve` pulls each gather's rows
  and `repeats` back through the `Sidebar` trait, which reads them from
  the cache. Once the binary renders the frames, `Serve` is better handed
  them with the gather, leaving the trait to `jumped`, `moving`, `reload`
  and `move_cursor`; the cache still gets `rows.N`, as `reload-sync` cats
  them. Left as is here, where a gather reads only the rows it shows.
- The Mac has the binary installed from this slice's last code commit.
  mgmt-1 needs it after pulling (`Readme.md`, from `~/dotfiles`):
  `cargo install --locked --path claude-sidebar-loop --root ~/.local`.
  Without it a sidebar starts with no loop: fzf stays on its first rows
  and the log says `command not found`. mgmt-1 was only built into a
  scratch root; its fzf (0.58) is too old to run the sidebar anyway.
- The checks behind fbb2842 and c6b0274 (a fake fzf logging posts, and
  the stage profile extended for the binary) are scratchpad scripts, not
  in the repo, so their numbers cannot be re-taken from it. Say if they
  should be shipped.
