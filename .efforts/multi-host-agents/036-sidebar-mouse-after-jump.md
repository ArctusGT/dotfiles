## The sidebar keeps its mouse through a jump

**Category:** bug
**Summary:** A click or Enter in the sidebar runs the jump inside fzf's silent execute, and a frame posted to fzf while a long one runs makes fzf turn mouse tracking off for good once it returns; the jump runs in the background so the execute returns at once, and the reproduction ships as a check
**Status:** ready-for-human
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

Measured further on a private tmux while implementing: fzf writes the
four sequences after the execute's last output, as it returns, so
nothing the command writes can undo them. It does so only for an
execute of about a second or more (posts during a 0.8 s execute leave
the flags at `111`, during a 1.0 s one at `000`), which a local jump
reaches when it opens a background session. A silent execute that puts
its command in the background, and a non-silent execute such as
`pin`'s, keep the mouse under the same posts. tmux holds one tracking
mode, the last written, and fzf's is button events.

**Desired behavior:**
- A jump from the sidebar, by click or Enter, leaves the sidebar's
  mouse working, every time, with a Clawd moving in the list while it
  runs, however long the jump takes
- fzf's silent execute puts the jump in the background and returns at
  once, so no execute is long enough for a post to turn the mouse off
- A check in the crate's checks reproduces fzf's behaviour on a private
  tmux with the sidebar's own actions, as the record of why the
  workaround exists, and drives a throwaway sidebar through repeated
  click jumps held past a second with a moving Clawd, reading the
  pane's mouse flags after each; it answers `--help`

**Key interfaces:**
- The sidebar's fzf bindings in `claude-sidebar`
- The checks directory and `lib.sh`, which run a sidebar in a throwaway
  tmux with its own TMPDIR

**Acceptance criteria:**
- [x] The shipped check shows fzf alone dropping the mouse under posts
      during a silent execute, and the sidebar keeping it through 30
      click jumps with a working session's Clawd running, on the Mac
- [ ] The Maintainer uses the sidebar for a session of work, jumping by
      click locally and to mgmt-1, with the mouse working throughout
- [x] The row comparison and the end-to-end check still pass

**Out of scope:**
- A fix in fzf itself; the reproduction is the material for a report
  upstream, which the Maintainer may file
- Pins, by `alt-p` or `Alt+Shift+N`: `alt-p`'s non-silent execute keeps
  the mouse under posts, and tmux runs `Alt+Shift+N` outside fzf
- Holding the loop's posts while a jump moves, or re-asserting the
  mouse from the command fzf ran: neither can stop a drop fzf writes as
  its execute returns
- Any change to what a jump does once it has the mouse

**For the Maintainer:**
- fzf no longer waits on a jump, so a second click during a slow one
  starts a second jump beside it, where fzf used to hold the click until
  the first returned. Each marks its own `moving.<pid>` and the last
  `jumped` written wins, and the loop follows the session actually on
  show, so the code review judged it benign; nothing checks overlapping
  clicks
- The end-to-end check's "four numbered entries" had failed since
  e4b5328 grouped by repo, its stub host's sessions carrying none;
  4f878b2 gave them one each so the last criterion could be met
