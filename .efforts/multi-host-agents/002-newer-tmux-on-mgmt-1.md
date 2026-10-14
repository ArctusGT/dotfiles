## Newer tmux on mgmt-1

**Category:** bug
**Summary:** Replace mgmt-1's tmux snapshot, which aborts on `capture-pane`, with tmux 3.5 or newer
**Status:** ready-for-human
**blocked-by:** [multi-host-agents/001-non-login-shell-env]

**Current behavior:**
mgmt-1 runs EL10's `tmux-3.3a` git snapshot, reporting `next-3.4`; dnf has
nothing newer. Any `capture-pane` aborts the whole server (SIGABRT in
`cmd_capture_pane_exec`), which ended every session on mgmt-1 once already.
The tmux config works around it by unbinding the theme's `F` and `U` helpers
when `#{version}` matches `next-*`, and agents are told never to run
`capture-pane` there. workmux calls `capture-pane`, so it cannot run on mgmt-1.

Reproduction, safe because it uses its own socket:
`tmux -L t -f /dev/null new -d` then `tmux -L t capture-pane -p`.

**Desired behavior:**
mgmt-1 runs a tmux 3.5 or newer, built for the `sysadmin` user without
replacing the system package. Every way tmux gets started on mgmt-1 (an
interactive shell, `ssh-tmux`, the oh-my-zsh tmux plugin) uses the new binary.
The live server is replaced at a time the Maintainer chooses: a new client
cannot talk to the old server, so the switch ends every running session.

**Key interfaces:**
- Build dependencies come from dnf (`gcc`, `make`, `libevent-devel`,
  `ncurses-devel`, `bison` or a release tarball that needs none); installing
  them needs the Maintainer's sudo
- The install prefix is the user's own, whose `bin` slice 001 puts first on
  PATH for every shell
- The `next-*` guard in the tmux config stays harmless on a newer version, as
  it only matches the snapshot

**Acceptance criteria:**
- [x] `tmux -V` on mgmt-1 reports 3.5 or newer, from an interactive shell and
      through `ssh mgmt-1 'tmux -V'`
- [x] The safe reproduction above prints the pane instead of
      `server exited unexpectedly`
- [x] Inside an `ssh-tmux` session, `#{version}` is the new version
- [x] The theme's `prefix F` is bound again on mgmt-1
- [x] `ssh-tmux` sessions, the Claude status marker and mouse copy still work
      from the Mac after the switch

**Out of scope:**
- Removing the system tmux package
- Removing the `next-*` guard or the `capture-pane` warning in AGENTS.md; a
  follow-up can do that once no host runs the snapshot
- workmux itself (slice 003)
