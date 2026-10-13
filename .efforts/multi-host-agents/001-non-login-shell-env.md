## Non-login shells get the interactive environment

**Category:** bug
**Summary:** Commands run as `ssh host '<cmd>'` see the same PATH, locale and editor as an interactive shell
**Status:** ready-for-closure
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
- [x] `ssh mgmt-1 'command -v claude; echo "$LANG $EDITOR"'` prints a path
      under `~/.local/bin` and a UTF-8 locale with `nvim`
- [x] `ssh mgmt-1 'echo $PATH'` lists `~/.local/bin` once, before `/usr/bin`
- [x] An interactive shell on each host prints the same PATH as before, apart
      from ordering, with no duplicates
- [x] `zsh -c 'echo "$LANG $EDITOR"'` on the Mac prints the same values
- [x] mgmt-1's dotfiles are pulled and restowed, and its stow reports no
      conflicts

**Out of scope:**
- Removing `tmux -u` from `ssh-tmux` or the `mode-keys vi` pin: both stay as
  belt and braces
- Changing the login shell or the oh-my-zsh setup
- Installing anything on either host

## Comments

2026-10-14, structural review, beyond this slice's scope, for the Maintainer:
`/opt/homebrew/bin` is still prepended only by the interactive config, so a
non-interactive shell on the Mac does not find Homebrew binaries, including
`nvim`, which `EDITOR` now names. `ssh mac 'git commit'` would fail to open an
editor. Mac non-interactive use is rare, so this slice left it out. Fixing it
is one `/opt/homebrew/bin(N/)` entry at the front of the user dir list the
every-shell startup file now keeps, plus deleting the interactive config's
Homebrew block.

---

2026-10-14, code review, two judgement calls left for the Maintainer:

- `LANG` is only a default. A value the ssh client sends and sshd accepts
  (RHEL accepts `LANG` by default) passes through, so a client sending
  `LANG=C` gets a non-UTF-8 shell. The Mac sends its own UTF-8 locale, so
  this holds today; forcing UTF-8 would override a terminal's own choice.
- `user_path` stays set as a plain variable in every zsh, scripts included,
  because the interactive config re-applies it. A script assigning its own
  `user_path` would only change its own copy, so the risk is a confusing name
  collision rather than a broken PATH.

Re-verified on mgmt-1 at the reviewed version (7795d85), where Cargo's bin
now reaches non-interactive shells too; 227bb7b had sourced `~/.cargo/env`,
which mgmt-1 does not have.

---
