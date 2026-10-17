## A Clawd beside each entry

**Category:** enhancement
**Summary:** Each sidebar entry gets Clawd, Claude Code's three-line block-character mascot, coloured by the session's state
**Status:** ready-for-closure
**blocked-by:** []

**Current behavior:**
An entry is three lines of text: number, repo; name; state, time, subagents
and context. The sidebar is 34 columns wide.

**Desired behavior:**
Clawd's three banner lines sit to the left of each session entry's three
lines, with the entry's number beside him. His colour shows the session's
state (working, waiting, idle), and a background entry's Clawd is dimmed with
the rest of it. Headers have no Clawd.

The sidebar widens by Clawd's width, so names, paths and the third line keep
the room they have now.

The Maintainer chose a three-line Clawd per entry over a one-character
glyph. The hat this slice first asked for traced to a claim, that
`tengu_birthday_hat` was a hat Claude Code draws: no Claude Code holds one,
and the hats the Maintainer had seen come from the third-party claude-pet
plugin. The Maintainer dropped the hat and its flag plumbing.

**Key interfaces:**
- Clawd's banner art, as Claude Code draws it, rather than an invented
  likeness
- The sidebar's width, set where `tmux-frame` splits the frame and where the
  frame keeps it on resize

**Acceptance criteria:**
- [x] Every session entry shows a three-line Clawd beside its three lines,
      coloured by its state, dimmed for a background entry
- [x] Entry text is cut no shorter than before the slice
- [x] The Readme describes the icon and its colours

**Out of scope:**
- The context bar (slice 011)
- Hats, Claude Code's or the sidebar's own
- Animating Clawd
