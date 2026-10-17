## Context use as a bar

**Category:** enhancement
**Summary:** Each sidebar entry ends in its own gap line: a full-width horizontal bar whose filled part shows context use in the tmux theme's gradient colours, with the percentage at its end, turning red at CLAUDE_CTX_WARN
**Status:** ready-for-closure
**blocked-by:** []

**Current behavior:**
An entry's third line ends `ctx 24%`, or `!ctx 62%` in red once at or past
`CLAUDE_CTX_WARN` (default 50). An entry with no status-line record yet shows
nothing.

**Desired behavior:**
Each session entry gets a fourth line in place of the blank row fzf's
`--gap` draws between entries: a horizontal bar across the sidebar's width
with the percentage at its end, e.g. `━━━━━━┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈ 24%`. The
filled share is a heavy line coloured by where it reaches along the theme's
gradient palette, green first, since a full context is the bad end; the rest
is a light line in the gap's grey. At or past `CLAUDE_CTX_WARN` the filled
part and the number are red, as `!ctx` was. An entry with no record shows
the line all grey, with no number. Headers keep a plain gap. The third line
loses its `ctx` text, so it no longer overflows the sidebar.

The Maintainer chose this over a short `◼◻◻◻` bar on the third line, after
fzf's `--gap-line` proved to be one string for every gap, unable to carry a
per-entry value. Bar plus percentage, red at the warn level, still holds.

**Key interfaces:**
- fzf's `--gap`, dropped, and the sidebar's own last line in its place;
  `--highlight-line` may run the focused entry's shading across it
- The theme's `tmux_conf_battery_bar_palette` gradient: the colours to
  match. The sidebar reads nothing from tmux at runtime; the colours are
  carried as the sidebar's own
- The sidebar's `format`, which builds each entry's lines, and the
  `context.used_pct` field `claude-sessions` reports
- `CLAUDE_CTX_WARN`, unchanged

**Acceptance criteria:**
- [x] An entry at 24% ends in a line about a quarter filled, in the
      theme's gradient colours, with `24%` at its end
- [x] An entry at or past `CLAUDE_CTX_WARN` shows its filled part and
      number red
- [x] An entry with no context record ends in an all-grey line
- [x] Entries stay visibly separated, with no blank row between them, and
      the third line fits the sidebar's width, `background` included
- [x] The Readme describes the line where it described the bar

**For the Maintainer's look:**
- Each filled cell takes the gradient colour of its own position along the
  bar, so below the default warn of 50 the fill reaches yellow at most. The
  other reading of "coloured by where it reaches", the whole fill in the one
  colour its end sits on, is a small change if preferred.
- A heading's blank row is a braille blank (U+2800), since fzf drops a
  trailing line of spaces. The last entry of one host now meets the next
  host's heading with no blank row, its line doing the separating.
- `--highlight-line` is not added; the focused entry shades as before.

**Out of scope:**
- The Clawd icon (slice 012)
- Changing what `claude-sessions` or the status line record
