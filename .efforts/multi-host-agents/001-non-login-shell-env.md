## Non-login shells get the interactive environment

**Category:** bug
**Summary:** Commands run as `ssh host '<cmd>'` see the same PATH, locale and editor as an interactive shell
**Status:** ready-for-agent
**blocked-by:** none

**Current behavior:**
A command passed to ssh runs in a non-interactive, non-login zsh, which never
reads the interactive startup file. On mgmt-1 that shell has no `LANG`, no
`EDITOR`, and no `~/.local/bin` on PATH. Each gap has already surfaced once:

- tmux started that way assumed a non-UTF-8 terminal and drew icons as `_`;
  `ssh-tmux` works around it with `tmux -u`.
- tmux started that way chose emacs copy-mode keys, where a mouse drag never
  copied; the tmux config now pins `mode-keys vi`.
- `ssh mgmt-1 claude` reports `command not found`, because the native Claude
  binary lives in `~/.local/bin`.

The Mac has the same split, but is rarely reached over ssh.

**Desired behavior:**
Every zsh, including the non-interactive one ssh starts for a command, gets
`~/.local/bin` (and `~/.cargo/bin` where present) ahead of the system PATH, a
UTF-8 `LANG`, and `EDITOR=nvim`. The interactive config keeps working
unchanged, and PATH does not gain duplicate entries when an interactive shell
also sets it.

**Key interfaces:**
- zsh's startup file that every zsh reads, interactive or not, as opposed to
  the interactive-only one that holds this today. It belongs in the `zsh` stow
  package so both hosts get it.
- `ssh-tmux`'s remote command: once this lands, `tmux` there resolves through
  the new PATH, which slice 002 relies on to pick up a newer tmux.

**Acceptance criteria:**
- [ ] `ssh mgmt-1 'command -v claude; echo "$LANG $EDITOR"'` prints a path
      under `~/.local/bin` and a UTF-8 locale with `nvim`
- [ ] `ssh mgmt-1 'echo $PATH'` lists `~/.local/bin` once, before `/usr/bin`
- [ ] An interactive shell on each host prints the same PATH as before, apart
      from ordering, with no duplicates
- [x] `zsh -c 'echo "$LANG $EDITOR"'` on the Mac prints the same values
- [ ] mgmt-1's dotfiles are pulled and restowed, and its stow reports no
      conflicts

**Out of scope:**
- Removing `tmux -u` from `ssh-tmux` or the `mode-keys vi` pin: both stay as
  belt and braces
- Changing the login shell or the oh-my-zsh setup
- Installing anything on either host
