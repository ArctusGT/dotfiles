## TokyoNight in nvim, lazygit and Claude Code

**Category:** enhancement
**Summary:** nvim's tokyonight uses the Night style, lazygit gets a TokyoNight theme in a new stow package, and Claude Code uses its terminal-palette theme, so all three match Ghostty
**Status:** ready-for-agent
**blocked-by:** []

**Current behavior:**
nvim loads tokyonight.nvim with the plugin's default style, not pinned to
Night. lazygit's config is an empty file outside the repo, so it shows its
default colours. Claude Code's settings set `"theme": "dark"`, its own
palette rather than the terminal's.

**Desired behavior:**
- nvim's tokyonight style is pinned to `night`, so its background is
  Ghostty's
- lazygit's config lives in the repo as a stow package and gives lazygit
  TokyoNight Night colours for borders, selection and highlights, named by
  terminal palette where lazygit accepts it
- Claude Code uses its ANSI dark theme, which draws from the terminal's
  palette, in place of `dark`, on every host the `claude` package reaches
- All of this works on the Mac and on mgmt-1

The Maintainer chose TokyoNight Night and the terminal palette as the source.

**Key interfaces:**
- tokyonight.nvim's `style` option, in the plugin's spec under the `nvim`
  package
- lazygit's `gui.theme` settings; its config path differs by OS (macOS
  reads `~/Library/Application Support/lazygit/config.yml` unless
  `XDG_CONFIG_HOME` is set), so measure which path each host reads before
  choosing where the package stows it
- Claude Code's `theme` setting in the `claude` package's user settings; the
  exact name of its ANSI dark theme is to be confirmed from Claude Code's own
  `/theme` list

**Acceptance criteria:**
- [ ] nvim's background matches Ghostty's (`#1a1b26`) on the Mac and on
      mgmt-1
- [ ] lazygit shows TokyoNight colours on both hosts from the stowed config,
      and the Readme lists its package with the others
- [ ] Claude Code's prompt, borders and diffs take the terminal's palette
      after the settings change
- [ ] No other tool's config is changed

**Out of scope:**
- tmux (slice 001), the sidebar and fzf (slice 002)
- nvim plugins other than the colourscheme
- Claude Code's statusline content
