## The sidebar keeps its mouse through a jump

**Category:** bug
**Summary:** A click or Enter in the sidebar runs the jump inside fzf's silent execute, and a frame posted to fzf while that runs makes fzf turn mouse tracking off for good; the jump and pin commands re-assert it as their last act, the loop posts nothing while a jump is moving, and the reproduction ships as a check
**Status:** ready-for-agent
**blocked-by:**

**Current behavior:**
The sidebar's fzf binds `enter` and `left-click` to a silent execute of
`claude-sidebar jump`, and `alt-p` to an execute of `claude-sidebar pin`.
The loop posts `reload-sync` and `pos` to fzf's socket on each gather,
and a frame every eighth of a second while any Clawd moves. fzf 0.74.5,
the current Homebrew one, answers a post that arrives while a silent
execute is still running with a redraw that turns mouse tracking off
(`?1000l ?1002l ?1006l`) and bracketed paste off, and never turns them
back on. From then on tmux keeps every click and the sidebar reads as
dead to the mouse until the sidebar is restarted, while keys still work.

Measured by the Maintainer and the Agent on the live sidebar: tmux's
pane-output copy caught the four sequences inside one synchronized
redraw, twelve milliseconds before a local jump wrote its marker, and
no mouse-on after. Reproduced on a private tmux with fzf alone: posts
during a three-second silent execute produce the same sequences and
leave tmux's pane mouse flags at zero; the same posts while idle, or
the same slow execute with no posts, change nothing. Writing the three
enable sequences to the pane's tty restores the mouse, and fzf keeps
reading clicks, as it still believes the mouse is on.

**Desired behavior:**
- A jump or a pin from the sidebar, by click, Enter or `alt-p`, leaves
  the sidebar's mouse working, every time, with a Clawd moving in the
  list while it runs
- The loop posts nothing to fzf while a jump is moving, which the
  marker the jump already writes tells it; a frame due in that time is
  shown once the jump has moved
- The command fzf ran writes the enable sequences for button and any
  event tracking and SGR encoding to its own terminal as its last act,
  so a post that slipped in before that is undone, and a post after it
  cannot land inside the execute
- A check in the crate's checks reproduces fzf's behaviour on a private
  tmux with the sidebar's own actions, as the record of why the
  workaround exists, and drives a throwaway sidebar through repeated
  click jumps with a moving Clawd, reading the pane's mouse flags after
  each; it answers `--help`

**Key interfaces:**
- `claude-sidebar`'s `jump` and `pin` commands, which run with the
  sidebar's terminal when fzf runs them
- The loop's posting to fzf and the moving marker it reads
- The checks directory and `lib.sh`, which run a sidebar in a throwaway
  tmux with its own TMPDIR

**Acceptance criteria:**
- [ ] The shipped check shows fzf alone dropping the mouse under posts
      during a silent execute, and the sidebar keeping it through 30
      click jumps with a working session's Clawd running, on the Mac
- [ ] The loop sends no post between a jump's moving marker appearing
      and going, by the check reading fzf's socket log or the loop's
      stderr at a debug setting, whichever the implementation offers
- [ ] The Maintainer uses the sidebar for a session of work, jumping by
      click locally and to mgmt-1, with the mouse working throughout
- [ ] The row comparison and the end-to-end check still pass

**Out of scope:**
- A fix in fzf itself; the reproduction is the material for a report
  upstream, which the Maintainer may file
- Pins by `Alt+Shift+N`, which tmux runs outside fzf and which never
  touched the sidebar's terminal
- Any change to what a jump does once it has the mouse
