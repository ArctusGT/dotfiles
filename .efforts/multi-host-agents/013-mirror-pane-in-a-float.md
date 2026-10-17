## Mirror a pane in a floating pane that stays on screen

**Category:** enhancement
**Summary:** Option+dragging a pane onto a floating pane makes the float mirror it, and the float stays over the frame's right pane while the Maintainer moves to other windows, sessions and hosts, so instructions in one pane stay readable while acting in another
**Status:** ready-for-human
**blocked-by:** [multi-host-agents/009-cmd-arrows-between-panes]

**Current behavior:**
Ctrl+drag over the frame's right pane opens a floating pane in the frame
itself, through tmux 3.8's default `C-MouseDrag1Pane` binding, since the
frame binds no mouse key of its own for it. The float runs a shell on the
Mac, can be moved and resized, and stays on screen whatever the right pane
shows, mgmt-1 included. Option+drag from one pane onto another in the same
window swaps the two (slice 009). Nothing shows one pane's contents in a
second place.

**Desired behavior:**
Option+dragging a pane and releasing over a mirror float makes the float show
that pane's contents, live and read-only, while the pane itself stays where
it was and keeps working. The float stays on screen over the frame's right
pane whichever window, session or host the right pane moves to, so the
Maintainer can read a Claude session's instructions while working in the
window those instructions are about. The source pane can be on the Mac or on
mgmt-1. Dropping another pane on the float switches what it mirrors. Closing
the float stops the mirror and touches nothing else.

The mirror is the frame float the Maintainer already opens with Ctrl+drag,
since only the frame sits above whatever the right pane shows.

**Key interfaces:**
- tmux 3.8 floating panes in the frame server: `new-pane` with size and
  position, `move-pane -M` and the border's resize drag, the border controls
  (control 9 kills, control 7 tiles), `kill-pane`
- The drop: the drag starts in the usual tmux (or mgmt-1's, through the
  relay) and ends over a pane of the frame. The usual tmux records the drag's
  start pane (009's `@drag_src`); the frame's `M-MouseDragEnd1Pane` sees the
  float under the pointer and asks for that start. How a drag starting in a
  relay window names mgmt-1's pane is to be measured
- The mirror itself: a read-only view of another server's pane, for example
  a refresh loop over `capture-pane -p -e` (over the sidebar's shared ssh
  connection for mgmt-1), or a read-only nested client (`attach -r`). Pick by
  measurement: redraw cost, flicker, and what happens on resize. On mgmt-1,
  only ever `~/.local/bin/tmux`: EL10's `/usr/bin/tmux` aborts the server on
  any `capture-pane`
- The sidebar's own panes stay as they are; the float is a third pane of the
  frame window

**Acceptance criteria:**
- [ ] Option+dragging a Mac pane onto the float makes it mirror that pane;
      output in the source shows in the float within a second
- [ ] Option+dragging a pane in the Mac's `mgmt-1` window onto the float
      mirrors that mgmt-1 pane
- [x] Typing while the float is focused does not reach the mirrored pane
- [x] Closing the float (its border's close control, or `kill-pane`) leaves
      the source pane and the right pane untouched
- [x] The Readme describes opening, filling and closing the mirror float

**Out of scope:**
- Typing into the mirrored pane through the float
- More than one mirror float at a time
- Mirroring outside the frame
- Floats inside the usual tmux's windows, which keep tmux's own behaviour

**Waiting on the Maintainer:**
The first two criteria passed with SGR mouse sequences fed to a scratch
frame's client, and the second through a scratch ssh relay client on mgmt-1.
They are ticked after a real Option+drag in Ghostty, onto a float made with
Ctrl+drag: one from a Mac pane, one from a pane in the Mac's `mgmt-1` window.

**Named for the Maintainer, beyond this slice:**
- Option+drag in an ssh-tmux window is now forwarded to the host's tmux, so
  it swaps the host's panes there as well; that is how the drag names a
  mgmt-1 pane for the drop.
- The relay client on a host is found by its termname (`tmux*`, first match),
  as `claude-sidebar` does, while `ssh-tmux` records it as `@relay_client`. A
  second nested client on the host could be taken instead. `ssh_opts`, `work`
  and the right-pane lookup are also copied from `claude-sidebar`; a shared
  sourced file, or folding `tmux-mirror` into `claude-sidebar`, would give
  them one home.
