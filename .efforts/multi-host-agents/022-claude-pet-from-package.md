## claude-pet from the `claude` package

**Category:** enhancement
**Summary:** The third-party `clawd-pet` mod (github.com/yuyongyan29-dev/claude-pet) is installed through the `claude` package's settings, so Clawd walks above the prompt in every Claude Code session on the Mac and mgmt-1
**Status:** ready-for-agent
**blocked-by:** []

**Current behavior:**
No plugin beyond the official marketplace is known or enabled on either host;
the package's `settings.json` sets neither `extraKnownMarketplaces` nor
`enabledPlugins`. The hats the Maintainer saw earlier came from this mod, run
outside the package.

**Desired behavior:**
- The package declares the `claude-pet-market` marketplace and enables
  `clawd-pet@claude-pet-market`, so both hosts get it from stow, with no
  per-host `/plugin install`
- The marketplace is pinned to the commit the Maintainer reviewed, if Claude
  Code's marketplace source takes a ref; an update is then a deliberate bump
  in the package
- A new interactive session shows Clawd above the prompt on both hosts;
  `claude -p` and background sessions are unaffected
- Inside tmux, and over ssh-tmux to mgmt-1, Clawd draws in block characters
  where the image protocol does not pass through; that is acceptable

The Maintainer wants the mod installed first, then to see whether its
animations fit the sidebar's Clawd (a later slice, if it fits).

**Key interfaces:**
- `claude/.claude/settings.json`: `extraKnownMarketplaces`, `enabledPlugins`,
  and whatever turns mods on, per code.claude.com's mods overview
- The mod's requirement: Claude Code 2.1.287+ in the terminal. Measured:
  the Mac runs 2.1.292, mgmt-1 2.1.302
- What the mod touches, read from its hooks at the reviewed commit
  (`92af7ce`, plugin 0.29.5): Claude
  Code's mod API only (`$.ui`, `$.store`, `$.fs`, `$.audio`, `$.env`), files
  under the Claude config dir (`clawd-pet-history/`, `stats-cache.json`), one
  process (`date +%H`), no network
- The Readme's `claude` package section

**Acceptance criteria:**
- [ ] The package's settings declare the marketplace, pinned if supported,
      and enable the plugin; `claude plugin validate` or `/plugin` lists it
      enabled on the Mac
- [ ] A new session on the Mac shows Clawd above the prompt, in a plain
      Ghostty tab and in a tmux pane
- [ ] The same on mgmt-1 after `git pull`, with no step there beyond the pull
- [ ] The Readme names the mod, where it comes from, and how to bump it

**Out of scope:**
- Bringing its animations or hats into `claude-sidebar`
- Its own settings (`/pet size`, hats, sound), which live in its store per
  host
