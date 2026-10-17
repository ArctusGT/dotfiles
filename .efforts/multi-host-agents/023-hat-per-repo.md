## A hat per repo on the sidebar's Clawd

**Category:** enhancement
**Summary:** Each sidebar entry's Clawd wears a pixel hat picked from its repo's name, so every session of one project looks alike on both hosts; the entry grows a line to hold it
**Status:** ready-for-agent
**blocked-by:** []

**Current behavior:**
An entry is four lines: Clawd's three (9 cells, quarter blocks, coloured by
state) beside the number and folder, the session name, and the state; then
the context bar. Sessions of one repo are told apart only by reading the
folder name.

**Desired behavior:**
- A one-line hat sits above Clawd's head: two pixel rows of the same
  quarter-block grid, 18 pixels wide, each cell keeping to two colours
- The hat comes from the repo's name, the same name `claude-sidebar attach`
  gives its tmux session, so a repo's subdirectories and worktrees, and the
  same repo on the Mac and on mgmt-1, wear one hat; outside a repo, the
  folder's name
- A stable hash of that name picks the shape and the colour. Hat colours
  avoid the state colours (orange, yellow, the idle grey) and red
- The entry grows to five lines: the number and repo name move up beside the
  hat, and the context bar stays last
- A background entry's hat is dimmed with the rest of it
- The shapes are drawn for this repo; none are copied from agents-deck, which
  carries no licence and whose idea (a hash of the folder picks the hat) this
  borrows

The Maintainer chose a fifth line over a compact glyph, and our own art over
agents-deck's.

**Key interfaces:**
- `claude-sidebar`'s jq row builder: `clawd($state)`, the entry's lines, and
  `paint`
- Everything that counts an entry's lines: `position` and its callers, fzf's
  scroll and click mapping
- The repo name, from `tmux_session_for`'s rule; for a remote entry it has to
  come from that host (`claude-sessions` there) or from the cwd the record
  already carries
- The hash: something both jq and the shell can compute the same way, or one
  of them only

**Acceptance criteria:**
- [ ] Two sessions in one repo, one in a subdirectory, wear the same hat; a
      session in another repo, most likely a different one
- [ ] `dotfiles` on the Mac and on mgmt-1 wear the same hat
- [ ] Entries are five lines; clicking, the frame's M-1..9, and Cmd+Up/Down
      at an edge still land on the right entry
- [ ] No hat uses a state colour or red, and a background entry's hat is dim
- [ ] At least eight shapes, each legible at the sidebar's size, as the
      Maintainer judges by eye

**Out of scope:**
- Choosing a hat by hand per repo
- A hat per session rather than per repo
- Anything else from agents-deck
