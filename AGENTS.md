@.agents/AGENTS.md

# dotfiles

Configs for zsh, tmux, nvim, sesh and Claude Code, symlinked into `$HOME` with
GNU Stow. Each top-level directory except the ones below is a stow package
whose contents mirror `$HOME`: `tmux/.tmux.conf.local` lands at
`~/.tmux.conf.local`. `Readme.md` lists the packages and the stow commands.
Used on macOS (the workstation) and on Linux hosts such as mgmt-1, so a change
has to work on both — mgmt-1 runs an older tmux (`next-3.4`) than the Mac.
Never run `tmux capture-pane` on mgmt-1: that build aborts and takes every
session with it. Read state with `display -p` or `list-windows -F` instead.

Not stow packages, and never stowed:
- `.agents/` — the shared kit (submodule).
- `.claude/` — this repo's own agent config: rules, skills and hooks for work
  *on* the dotfiles. Easy to confuse with `claude/.claude/`, which is the
  stow package holding the user-level `~/.claude/settings.json`.
- `docs/` — effort-tracker docs the kit's skills read.

Most rules under `.claude/rules/` and every skill under `.claude/skills/` are
symlinks into `.agents/`, the shared kit. Editing one edits the kit and changes
every project that links it. To change a rule for this repo only, replace the
symlink with a real file.

Neither `find` nor `grep -r` descends into them. Both report only the few real
files and no error, so a search reads as a clean answer when it has seen almost
nothing. Use `find -L` and `grep -R`.

Project-specific, and safe to edit in place: `AGENTS.md`,
`.claude/settings.json`, and everything in the stow packages.
