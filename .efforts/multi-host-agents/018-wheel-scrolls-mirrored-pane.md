## The wheel over a mirror float scrolls the mirrored pane

**Category:** enhancement
**Summary:** The mouse wheel over a frame float showing `tmux-mirror show` scrolls the pane it mirrors exactly as the wheel over that pane would, and the float shows the result
**Status:** ready-for-agent
**blocked-by:** []

**Current behavior:**
A mirror float is a frame pane respawned as `tmux-mirror show`. The frame's
default root bindings pass the wheel to the pane under it, so the wheel over a
mirror float scrolls the float's own history: the shell it ran before it was
respawned, not the mirrored pane.

**Desired behavior:**
- The wheel over a mirror float reaches the mirrored pane as if the pointer
  were over it: a program that takes the mouse (Claude Code, nvim) scrolls
  itself, and a shell's pane enters tmux copy-mode and scrolls its history
- The float redraws what the mirrored pane now shows, including copy-mode's
  scrolled view, and returns to the bottom when the pane leaves copy-mode
- This holds for a pane of the usual tmux and for a pane on an ssh-tmux host
  (mgmt-1)
- The wheel over any other float, the sidebar or the right pane behaves as
  now

The Maintainer chose scrolling the mirrored pane, as the wheel does over it,
over ignoring the wheel on a mirror float.

**Key interfaces:**
- The frame's `WheelUpPane` / `WheelDownPane` root bindings, and how a float
  is known to be a mirror (`tmux-mirror show` as its command, or an option
  `drop` sets) and which pane it mirrors
- tmux's own default wheel binding in the usual tmux, which already decides
  between passing the wheel to the program and entering copy-mode; reusing
  its decision keeps "exactly as over that pane" true
- `tmux-mirror show`'s read of the pane, now `capture-pane -p -e`: measure on
  the Mac's tmux and mgmt-1's tmux 3.8 whether it shows copy-mode's view, or
  needs `capture-pane -M`
- The mirror's ssh stream to a host, for a mgmt-1 pane

**Acceptance criteria:**
- [ ] Over a float mirroring a Claude Code pane, the wheel scrolls Claude
      Code's transcript, seen in the float and in the source pane alike
- [x] Over a float mirroring a shell pane, the wheel scrolls the shell's
      history in copy-mode, and scrolling back to the bottom leaves it
- [ ] Both hold for a mirrored mgmt-1 pane
- [x] The wheel over a float that is not a mirror scrolls that float, as now

**Out of scope:**
- Keys, clicks or selection in a mirror float; it stays read-only otherwise
- Scrolling a float's own history
