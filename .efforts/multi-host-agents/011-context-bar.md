## Context use as a bar

**Category:** enhancement
**Summary:** Each sidebar entry shows its context use as a bar in the tmux theme's battery-bar style, with the percentage beside it, turning red at CLAUDE_CTX_WARN as the text does now
**Status:** ready-for-agent
**blocked-by:** []

**Current behavior:**
An entry's third line ends `ctx 24%`, or `!ctx 62%` in red once at or past
`CLAUDE_CTX_WARN` (default 50). An entry with no status-line record yet shows
nothing.

**Desired behavior:**
The text becomes a short bar and the percentage, e.g. `◼◼◻◻◻ 24%`. The bar
uses the symbols the usual tmux's theme gives its battery bar and colours its
filled part along the theme's gradient palette. At or past `CLAUDE_CTX_WARN`
the bar and number are red, as `!ctx` is now. No record, no bar.

The Maintainer chose bar plus percentage, red at the warn level, over a bar
alone.

**Key interfaces:**
- The theme's `tmux_conf_battery_bar_symbol_full` / `_empty` and
  `tmux_conf_battery_bar_palette` settings: the look to match. The sidebar
  reads nothing from tmux at runtime for this; the symbols and colours are
  carried as the sidebar's own
- The sidebar's `format`, which builds each entry's lines, and the
  `context.used_pct` field `claude-sessions` reports
- `CLAUDE_CTX_WARN`, unchanged

**Acceptance criteria:**
- [ ] An entry at 24% shows a bar about a quarter filled and `24%`, in the
      theme's symbols and colours
- [x] An entry at or past `CLAUDE_CTX_WARN` shows its bar and number red
- [x] An entry with no context record shows no bar
- [x] The third line fits the sidebar's width with state, time and `+N`
      subagents beside the bar
- [ ] The Readme describes the bar where it now describes `!ctx`

**Out of scope:**
- The Clawd icon (slice 012)
- Changing what `claude-sessions` or the status line record
