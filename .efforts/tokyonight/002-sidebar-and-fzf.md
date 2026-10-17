## TokyoNight in the sidebar and fzf

**Category:** enhancement
**Summary:** The sidebar's Clawd, context line, headings and dimming, and fzf's pointer, highlight and accents everywhere it runs, take TokyoNight Night's colours from the terminal palette
**Status:** ready-for-agent
**blocked-by:** []

**Current behavior:**
`claude-sidebar` writes fixed 256-colour numbers: Clawd orange 173, waiting
yellow 220, idle grey 245, the context line's gradient 46 to 196 and its grey
59, red at the warn level. fzf, in the sidebar and in `sesh-pick` and the
shell, runs with its default colours, which `FZF_DEFAULT_OPTS` does not set.

**Desired behavior:**
- Clawd's state colours, the context line's fill, its unfilled grey and the
  warn red, the headings and the `unreachable` mark use TokyoNight Night,
  named by terminal palette number where one of the 16 fits
- The context line's gradient runs through TokyoNight's green, yellow,
  orange and red rather than the 256-colour cube
- fzf's pointer, current-line highlight, match highlight, borders, gutter and
  prompt use TokyoNight colours, once for every fzf the Maintainer runs (the
  sidebar, `sesh-pick`, the shell's fzf), and the sidebar's own options add
  only what is particular to it
- Dimmed background entries stay readable against TokyoNight's background

The Maintainer chose TokyoNight Night and the terminal palette as the source.

**Key interfaces:**
- The sidebar's `format`: Clawd's colours by state, the context line's
  palette and grey, and its warn red
- fzf's `--color` and `FZF_DEFAULT_OPTS`, set where every shell gets it
  (zsh's environment), on the Mac and on mgmt-1
- TokyoNight Night's palette, as Ghostty's bundled theme file states it

**Acceptance criteria:**
- [x] Every colour `claude-sidebar` writes is a terminal palette number or a
      hex value naming the TokyoNight colour it stands for
- [x] The context line runs green to red in TokyoNight's colours, and turns
      the warn red at `CLAUDE_CTX_WARN`
- [x] The sidebar's focused entry, pointer and match highlight use TokyoNight
      accents, and `sesh-pick` (`prefix f`) shows the same
- [ ] A background entry is still visibly dimmed
- [x] The Readme's sidebar section names no colour that is no longer used

**Out of scope:**
- tmux's bar and borders (slice 001)
- nvim, lazygit and Claude Code (slice 003)
- Changing what the sidebar shows or its layout

**For the Maintainer:**
- The gradient spans the whole bar, as the old one did, so below the
  default `CLAUDE_CTX_WARN` of 50 a line shows only green and yellow; orange
  and red show only with the warn level above 75. Spreading it up to `$warn`
  instead would change what a colour means, so it waits on a yes.
- Host headings are bold blue (palette 4), where they were bold default
  foreground; idle Clawd is TokyoNight's dark5 (#737aa2) rather than
  palette 8, which reads too dark under a background entry's dim.
