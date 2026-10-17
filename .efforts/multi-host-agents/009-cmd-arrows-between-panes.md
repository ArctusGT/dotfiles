## Cmd and arrows move between panes

**Category:** enhancement
**Summary:** Cmd+arrows move focus between the panes of a tmux window, and Cmd+Shift+arrows or an Option+drag move a pane, omarchy's Super bindings, on the Mac and through to mgmt-1's tmux
**Status:** ready-for-agent
**blocked-by:** []

**Current behavior:**
Moving between panes takes the prefix and a direction key. Ghostty binds
Cmd+arrows itself: Cmd+Left and Cmd+Right send line start and end, and
Cmd+Up, Cmd+Down and their Shift forms jump between shell prompts. None of
them reach tmux.

**Desired behavior:**
In the usual tmux, inside the frame or not, Cmd+Left/Right/Up/Down focus the
pane in that direction, and Cmd+Shift+Left/Right/Up/Down swap the focused pane
with its neighbour that way, keeping it focused. At a window's edge nothing
happens; slice 010 gives the edges a meaning. In an `ssh-tmux` window the keys
go on to the host's tmux and act on the host's panes, as `ctrl+left` and
`ctrl+right` do there now.

Holding Option and dragging from one pane to another in the same window
swaps the two, as Cmd+Shift+arrows do for neighbours. A plain drag still
selects text and a drag on a border still resizes.

Ghostty's defaults for these eight keys are given up. The Maintainer chose
Cmd over another modifier to match omarchy's Super habits.

**Key interfaces:**
- Ghostty's keybinds for `super+arrow_*` and `super+shift+arrow_*`: unbound,
  and either passed through or sent as a sequence tmux reads. Whether tmux
  3.8 recognises a Super modifier at all is unmeasured: measure that first,
  through the frame, which runs with `extended-keys always` in `csi-u` form
- The frame's root bindings: every key it does not bind reaches the right
  pane, so the frame needs none for this slice
- tmux's `M-MouseDrag1Pane` and `M-MouseDragEnd1Pane`, which Ghostty's
  Option-as-Alt makes reachable. Cmd cannot drive the drag: terminal mouse
  reports carry no Super bit, and Ghostty keeps Cmd+click for links. Whether
  the drag's end targets the pane under the pointer or the one it began in
  is unmeasured; if the latter, the drop pane comes from the pointer's
  position
- The `@ssh_tmux` window option, which the existing `C-Left`/`C-Right`
  bindings test to forward a key to the host

**Acceptance criteria:**
- [ ] In a Mac window split four ways, Cmd+arrows reach each pane from its
      neighbour, in the frame's right pane and in a plain Ghostty tmux
- [ ] Cmd+Shift+arrows swap the focused pane with its neighbour in that
      direction, and focus stays on the moved pane
- [ ] In the Mac's `mgmt-1` window, the same keys move between and swap
      mgmt-1's panes, and the Mac's window is unchanged
- [ ] Option+dragging from one pane and releasing over another in the
      same Mac window swaps them; a plain drag still selects text
- [ ] Cmd+arrows at a window's edge leave focus where it was
- [ ] The Readme's key table lists the new keys and says Ghostty's Cmd+arrow
      defaults are given up

**Out of scope:**
- Crossing a window's edge into the sidebar or another session (slice 010)
- Resizing panes
- Dragging a pane into another window or session, or joining it into a
  split rather than swapping
- Option+drag inside an `ssh-tmux` window
- Linux desktops: the Mac's Ghostty is the only terminal these keys come from
