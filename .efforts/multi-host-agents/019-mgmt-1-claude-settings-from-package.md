## mgmt-1's Claude settings from the `claude` package

**Category:** bug
**Summary:** mgmt-1's user-level Claude Code settings become the stowed `claude` package, so Claude Code there runs as it does on the Mac: the same theme, hooks and defaults
**Status:** ready-for-human
**blocked-by:** []

**Current behavior:**
On mgmt-1, `~/.claude/settings.json` is a real file rather than the package's
symlink, so changes to the package do not reach it. Against the package it
adds `model: opus[1m]`, `effortLevel: high`, and empty `enabledPlugins` and
`extraKnownMarketplaces`; it lacks `awaySummaryEnabled: false` and the
`claude-git-guard` PreToolUse hook on Bash, and sets `theme: dark` where the
package has `dark-ansi`. The guard's script is stowed there already, so only
the hook that calls it is missing.

**Desired behavior:**
- `~/.claude/settings.json` on mgmt-1 is the `claude` package's, by stow, as
  on the Mac
- Model and effort are Claude Code's defaults on both hosts, since the
  package sets neither; nothing per-host overrides them
- The guard denies `git reset`, `git stash` and `git restore` from Claude
  Code on mgmt-1

The Maintainer wants user-level settings identical wherever they work, kept
empty until something needs them, with per-repo config coming from the agent
kit; mgmt-1's extra keys are a holdover.

**Key interfaces:**
- The `claude` stow package, and the Readme's stow steps
- mgmt-1's `~/.claude/settings.json`, which stow refuses to replace while it
  is a real file; removing it is the Maintainer's step there, through
  mgmt-1's Claude agent
- `~/.claude/settings.local.json` exists on neither host; it stays absent

**Acceptance criteria:**
- [ ] On mgmt-1, `~/.claude/settings.json` is a symlink into the `claude`
      package
- [ ] A new Claude Code session on mgmt-1 draws in `dark-ansi` and runs
      Claude Code's default model and effort
- [ ] On mgmt-1, Claude Code is denied a `git stash` by `claude-git-guard`

**Out of scope:**
- Which model a subagent runs; that comes from each agent's definition in
  the agent kit
- Project-level `.claude/settings.json` in any repo
