## A window's edges lead into the sidebar

**Category:** enhancement
**Summary:** Cmd+Left from a window's leftmost pane focuses the sidebar, and Cmd+Up/Down past a window's top or bottom pane move to the sidebar's previous or next entry, so Cmd+arrows travel across every Claude session on every host
**Status:** ready-for-agent
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
- [ ] The Readme's key table describes the edges

**Out of scope:**
- Cmd+Right from a window's rightmost pane (nothing to its right)
- Unnumbered (background) entries: crossing skips them, as `M-N` does
- Changing `M-s`, `M-N`, click or Enter
