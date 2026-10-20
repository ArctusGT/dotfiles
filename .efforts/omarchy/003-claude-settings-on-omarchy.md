## Claude settings on Omarchy

**Category:** enhancement
**Summary:** An Omarchy host stows the `claude` and `bin` packages, so Claude Code there has the Maintainer's settings, the sidebar's hooks and status line, and `claude-git-guard`
**Status:** ready-for-agent
**blocked-by:**

**Current behavior:**
The Omarchy workstation's user-level Claude settings are a real file
holding only `"theme": "auto"`, so stow refuses to link the `claude`
package over it. The package's settings set `"theme": "dark-ansi"`, the
model, the subagent model, the sidebar's hooks and status line, and the
`claude-git-guard` hook, each calling a program under `~/.local/bin`. The
`bin` package still carries the sidebar's programs beside
`claude-git-guard`, `sesh-pick` and `espidf-index`, until agent-sidebar's
switch-over removes them.

**Desired behavior:**
- On an Omarchy host the `claude` and `bin` packages stow; the existing
  settings file gives way to the package's, and the Maintainer decides at
  review whether the theme stays `dark-ansi` there or follows the terminal
- Every hook and the status line find their program once agent-sidebar is
  installed as `workstation`
- `bin` is stowed on an Omarchy host only after agent-sidebar's
  switch-over, so no sidebar program is linked from this repo there
- Readme lists these packages for an Omarchy host

**Key interfaces:**
- The user-level Claude settings, and the programs its hooks call
- agent-sidebar's installer, a prerequisite listed in Readme

**Acceptance criteria:**
- [ ] `stow -n claude bin` on the Omarchy workstation reports no conflict
      after the existing settings file is moved aside
- [ ] A Claude session on the Omarchy workstation writes its state file
      and its status line shows context use
- [ ] `claude-git-guard` denies a `git stash` run by a session there
- [ ] `find -L ~/.local/bin -maxdepth 1 -type l` lists nothing on the
      Omarchy workstation

**Out of scope:**
- The hooks' behaviour, which is agent-sidebar's
- Other packages on Omarchy: zsh, nvim, lazygit, ghostty
