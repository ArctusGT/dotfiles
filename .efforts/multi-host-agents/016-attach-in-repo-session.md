## Open a background session in its repo's tmux session

**Category:** enhancement
**Summary:** Selecting a background entry opens its `claude attach` window in a tmux session named for the session's repo, created if absent, on the session's own host, so mgmt-1's layout matches the Mac's one-session-per-repo and the window list says which repo is on screen
**Status:** ready-for-human
**blocked-by:** []

**Current behavior:**
`claude-sidebar attach` opens a background session's `claude attach` window
in whichever tmux session is on show: the frame's right pane on the Mac, the
relay client's on a remote host. On mgmt-1, where most Claude work runs as
background jobs, every one lands in one tmux session as a window named
`claude`, so nothing on screen tells them apart, and a select that should
reach one conversation lands in a window of another. On the Mac, sesh gives
each repo its own tmux session with one interactive `claude`, and the
mix-up rarely happens.

**Desired behavior:**
A background session opens in the tmux session named for its repo, the last
component of its working directory (`slurm-infra`, `dotfiles`), on its own
host. If that tmux session does not exist, it is created there, detached, in
that directory. The shown client then switches to it, as for any other
entry. On the Mac the name matches the one sesh gives the same directory, so
an attach window joins the repo's existing sesh session rather than starting
a second one. Reusing an already-open attach window works as before,
wherever it is.

**Key interfaces:**
- `claude-sidebar attach ID [CLIENT]`: chooses the tmux session from the
  session's `cwd` rather than CLIENT's current session
- The session's `cwd`, which `claude-sessions` already reports
- sesh's session names for a directory (its config's `name`, or the
  directory's basename), so a Mac attach window lands in the repo's sesh
  session; measure what sesh names a directory it has no entry for
- tmux session names cannot contain `.` or `:`; a directory with either
  needs the same rewriting sesh applies
- The remote side runs on the host through the sidebar's background ssh, as
  `attach` does now; a remote switch moves the relay client to the new
  session, which `jump` already does for panes in other sessions

**Acceptance criteria:**
- [x] Selecting a Mac background entry whose repo has a sesh session opens
      its window in that session and shows it
- [ ] Selecting a mgmt-1 background entry opens its window in a mgmt-1 tmux
      session named for its repo, creating the session when absent, and the
      Mac's `mgmt-1` window shows it
- [x] Two background sessions in different repos end in different tmux
      sessions; two in the same repo share one
- [x] A reselect still reaches the existing window rather than opening a
      second
- [x] The Readme's paragraph on background entries says where they open

**Out of scope:**
- Moving windows already open into their repo's session
- Interactive sessions started by hand
- A pane switching conversations inside Claude Code (slice 015)

**Left for the Maintainer:**
- "Named for its repo" is read as the last component of the cwd, as the
  description words it. A job started in a subdirectory (`slurm-infra/roles`)
  lands in `roles`, and one in a worktree (`<repo>/.claude/worktrees/<name>`)
  in a session of the worktree's name, one per job. The main repo's name is
  `basename "$(dirname "$(git -C "$cwd" rev-parse --path-format=absolute
  --git-common-dir)")"`. Should a session be named for that instead?
- Criterion 2 needs mgmt-1 to pull, then a click on a mgmt-1 background entry
  in the sidebar, watching the Mac's `mgmt-1` window. The piped run of this
  copy's `attach` there created `slurm-infra` and `iot-infra` as expected, but
  the relay client's switch was not watched.
- `attach` no longer takes CLIENT, which Key interfaces still lists: with
  the tmux session chosen from the cwd nothing read it, and `jump` stops
  passing it on both sides. mgmt-1 running the old copy ignores the
  missing argument until it pulls.
