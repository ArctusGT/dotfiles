## A bandana per session on the sidebar's Clawd, in octants

**Category:** enhancement
**Summary:** The sidebar draws Clawd in Unicode octants wearing a bandana whose colours come from the session, in place of 023's per-repo hats, and the entry returns to four lines
**Status:** ready-for-human
**blocked-by:** []

**Current behavior:**
`claude-sidebar` draws Clawd in quarter blocks (2x2 pixels a cell) under a
per-repo hat, five lines an entry, from the hat work of the cancelled 023.

**Desired behavior:**
- Clawd is drawn in octants (2x4 pixels a cell), 16 pixels wide and 12 tall:
  three lines, then the context bar, four lines an entry as before 023
- He wears a bandana in place of his head-top row: a main colour and three
  stripe colours at one end. A one-colour ribbon hangs from it; standing,
  a tail on the right
- The colours come from the session id, so each session's bandana is its
  own and stays the same across refreshes and on both hosts
- The body stays coloured by state, as now; bandana colours keep clear of
  the state colours (orange, yellow, the idle grey)
- Every pose keeps each 2x4 cell to two colours at most, the background
  counting as one, so no pixel is dropped
- A background entry is dimmed, bandana included
- The hats, and the shapes and hashing that pick them, are removed

The Maintainer chose octants over quarter blocks, and a bandana per session
over 023's hat per repo, after `prototype-clawd-run.py` on the branch
`prototype/clawd-bandana` (c7a2438), whose STAND frame and octant table are
the starting point.

**Key interfaces:**
- `claude-sidebar`'s jq row builder: `clawd_px`, `hats`, `picture`, and the
  entry's lines
- The octant table: 230 characters at U+1CD00, named BLOCK OCTANT-n, plus
  26 patterns Unicode encodes elsewhere (quadrants, quarter blocks, U+1CEAx,
  U+1FBE6/7); the prototype builds it from Python's unicodedata
- A session's colours from a stable hash of its id, computed in jq

**Acceptance criteria:**
- [x] Each session's bandana differs from the others on show and stays the
      same across sidebar restarts
- [x] The standing pose has no cell holding more than two colours, checked
      by a script that counts them, as the prototype's clash count does
- [x] Entries are four lines; clicks, M-1..9 and Cmd+Up/Down at an edge land
      on the right entry
- [x] A background entry's Clawd and bandana are dim
- [ ] The Maintainer judges it by eye in the frame, on a Mac and a mgmt-1
      entry

**Out of scope:**
- Any animation (025, 026)
- A pose for waiting other than standing
- Whether a home directory's session should go bare: it wears a bandana
  like any other until the Maintainer says otherwise

## Comments

For the Maintainer, from review, alongside the look in the frame:
- Bandanas differ as a whole, but the main colour, half the bandana, comes
  from 9 shades, so with more than 9 sessions on show mains repeat (11 on
  show: three share blue1, two red, two teal). Guaranteeing distinct mains
  would mean picking against the other entries, which a stable hash of the
  id alone cannot do. Is a whole-bandana difference enough?
- The fg shade (#c0caf5) may read close to the idle grey once dimmed on a
  background entry.
- A red bandana can sit beside a red context bar past `CLAUDE_CTX_WARN`.
