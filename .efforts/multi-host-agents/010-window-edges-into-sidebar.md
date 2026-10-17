## A window's edges lead into the sidebar

**Category:** enhancement
**Summary:** Cmd+Left from a window's leftmost pane focuses the sidebar, and Cmd+Up/Down past a window's top or bottom pane move to the sidebar's previous or next entry, so Cmd+arrows travel across every Claude session on every host
**Status:** ready-for-closure
**blocked-by:** [multi-host-agents/009-cmd-arrows-between-panes]

**Current behavior:**
After slice 009, Cmd+arrows move between the panes of one window and stop at
its edges. Reaching the sidebar takes `M-s`, and another session takes a click
or `M-N`.

**Desired behavior:**
- Cmd+Left in a window's leftmost pane focuses the sidebar, as `M-s` does.
  In the sidebar, Cmd+Up and Cmd+Down move its cursor, and Cmd+Right shows the
  entry under the cursor in the right pane and focuses the right pane. Enter
  and a click are unchanged
- Cmd+Down in a window's lowest pane shows the sidebar's next numbered entry,
  and Cmd+Up in its top pane the previous one, in the sidebar's order across
  hosts, as `M-N` would for that entry. Past the last or before the first
  entry, nothing happens
- The same holds in an `ssh-tmux` window, judged by mgmt-1's active pane:
  a key mgmt-1's pane can still move within goes to mgmt-1, and one at its
  edge moves the sidebar
- Outside the frame there is no sidebar: the edges stay inert

The Maintainer chose "next sidebar entry" over "next tmux window" for the
vertical edges, and for Cmd+Left to enter the sidebar.

**Key interfaces:**
- The usual tmux's edge formats (`pane_at_left`, `pane_at_top`,
  `pane_at_bottom`) decide between moving within the window and crossing out
- `claude-sidebar pick N`, or an equivalent for "next" and "previous" relative
  to the entry on show (field 6 of the sidebar's rows marks it)
- The frame server (`tmux -L frame`) holds the sidebar pane; the usual tmux
  reaches it to focus the sidebar
- mgmt-1's tmux cannot reach the Mac's frame. For a relay window the Mac asks
  mgmt-1 whether its active pane is at the edge, over the sidebar's shared ssh
  connection, before forwarding the key or moving the sidebar

**Acceptance criteria:**
- [x] Cmd+Left in the leftmost pane of a Mac window focuses the sidebar;
      Cmd+Up/Down there move the cursor and Cmd+Right shows that entry and
      returns focus to the right pane
- [x] Cmd+Down in the lowest pane shows the next numbered entry and moves
      the sidebar's cursor to it; Cmd+Up in the top pane the previous one
- [x] From the last entry on the Mac, Cmd+Down reaches mgmt-1's first entry,
      and back with Cmd+Up
- [x] In the Mac's `mgmt-1` window, Cmd+Up/Down move between mgmt-1's panes
      and cross to the neighbouring entry only from mgmt-1's top or lowest
      pane
- [x] The Readme's key table describes the edges

**Out of scope:**
- Cmd+Right from a window's rightmost pane (nothing to its right)
- Unnumbered (background) entries: crossing skips them, as `M-N` does
- Changing `M-s`, `M-N`, click or Enter

## Comments

- Every criterion was measured by sending the keys through the frame's client
  with `send-keys -K`, not by pressing Cmd. Real presses are the Maintainer's
  to confirm, on the Mac and through the `mgmt-1` window.
- mgmt-1 needs a `git pull` before its own tmux has the new bindings and
  `claude-sidebar edge` (there, at its own edges, it finds no frame and does
  nothing).
- For the Maintainer, beyond this slice: the vertical edges count from the
  pane at the edge, so from a shell split below a Claude pane Cmd+Down does
  nothing. Counting from the window's entry instead would need a rule for a
  window holding two.
- For the Maintainer, beyond this slice: in an ssh-tmux window outside the
  frame, Cmd+Left/Up/Down now go to the host through a background
  `claude-sidebar edge` rather than straight from the binding, so presses
  faster than a fork could arrive out of order. Testing the client's
  `tmux*` termname inline would keep plain Ghostty on the old path, at the
  cost of a third branch in three bindings.
- For the Maintainer: inside the frame, each Cmd+Left/Up/Down in the `mgmt-1`
  window waits on an ssh round trip, and a crossing into a mgmt-1 entry
  switches mgmt-1's client in the background, so a held key there can read
  mgmt-1's pane before the switch lands and repeat an entry. A held key in a
  Mac window walks the list (measured, two presses 150ms apart).
