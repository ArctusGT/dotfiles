## TokyoNight in tmux's bar and borders

**Category:** enhancement
**Summary:** The tmux bar and pane borders, in the usual tmux, the frame and mgmt-1's tmux, take TokyoNight Night's colours, and Ghostty's theme moves into the repo as the one source of the palette
**Status:** ready-for-agent
**blocked-by:** []

**Current behavior:**
Ghostty shows TokyoNight (Night), set by `theme = TokyoNight` in the untracked
Application Support config. The usual tmux's bar uses oh-my-tmux's own hex
palette (near-black, yellow, pink, light blue), which does not match. The
frame's borders are `colour8`. mgmt-1's tmux, shown through the Mac's relay
window, uses the same tmux config as the Mac.

**Desired behavior:**
- Ghostty's theme is set in the repo's `ghostty` package as TokyoNight Night,
  the palette every other tool refers to
- The bar's segments, window list, the `@claude_wait` mark, and the active and
  inactive pane borders use TokyoNight Night's colours, in the usual tmux and
  in the frame, on the Mac and on mgmt-1
- Colours are named by terminal palette number (`colour0`..`colour15`) where
  one of the 16 fits, so they follow Ghostty's palette; a hex value only where
  the bar needs a shade the 16 lack (such as TokyoNight's darker background
  for the bar), with the TokyoNight name it stands for beside it

The Maintainer chose TokyoNight Night, the variant Ghostty already shows, and
the terminal palette as the source over hex in every tool.

**Key interfaces:**
- `tmux_conf_theme_colour_*` and the theme's other `tmux_conf_theme_*` style
  settings in the oh-my-tmux local config; the battery bar's palette as well
- The frame's `pane-border-style` and `pane-active-border-style`
- Ghostty's `theme` option in the `ghostty` package; the Application Support
  config loads after it and wins on a clash, so its own `theme` line has to
  go (the Maintainer's hand step, since that file is untracked)
- TokyoNight Night's palette, as Ghostty's bundled `TokyoNight Night` theme
  file states it; `ghostty +show-config` shows the loaded values

**Acceptance criteria:**
- [ ] `ghostty +show-config` reports TokyoNight Night's background and
      palette with the theme set only in the `ghostty` package
- [x] The usual tmux's bar, window list and pane borders show TokyoNight
      colours, with no oh-my-tmux default yellow or pink left
- [x] The frame's borders match the usual tmux's
- [ ] mgmt-1's bar, seen through the Mac's `mgmt-1` window, matches the
      Mac's after a pull
- [x] Every hex value left in the tmux configs names the TokyoNight colour it
      stands for

**Out of scope:**
- The sidebar's own colours and fzf (slice 002)
- nvim, lazygit and Claude Code (slice 003)
- Light or other TokyoNight variants, and switching between them
