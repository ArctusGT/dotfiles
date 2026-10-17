## Cmd+Up/Down at an edge steps from any pane of an entry's window

**Category:** bug
**Summary:** Cmd+Up/Down at a window's top or bottom edge moves to the previous or next sidebar entry from any pane of an entry's window, not only from the Claude pane itself
**Status:** ready-for-human
**blocked-by:** []

**Current behavior:**
`claude-sidebar edge up|down` calls `step`, which finds the current entry by
looking the edge pane up in the sidebar's rows. Rows carry only each entry's
Claude pane, so from any other pane of that window (a shell split under
Claude) the lookup finds nothing and the key does nothing. In `dotfiles`,
Cmd+Down from Claude reaches the zsh pane below, and the next Cmd+Down stops
there instead of moving to the next entry.

**Desired behavior:**
- At the edge, a pane that is not an entry steps from the entry in its own
  tmux window
- The Claude pane itself steps as now
- Holds for a window of the usual tmux and for an ssh-tmux host's window,
  where the host's active pane decides, as now
- A window with no entry in it still does nothing

**Key interfaces:**
- `claude-sidebar`'s `edge` and `step`
- For an ssh-tmux window, the one ssh query `edge` already makes for the
  host's active pane

**Acceptance criteria:**
- [x] On the Mac, Cmd+Down from the shell pane under an entry's Claude pane
      shows the next entry
- [x] The same from a shell pane under a mgmt-1 entry's Claude pane
- [x] Cmd+Up/Down from a Claude pane at the edge behaves as before

**Out of scope:**
- Cmd+Left into the sidebar, which does not look up an entry
- Windows holding more than one Claude pane: the first entry found is used
