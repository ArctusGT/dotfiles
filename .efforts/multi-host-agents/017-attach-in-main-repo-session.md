## Open a background session in its main repo's tmux session

**Category:** enhancement
**Summary:** A background session's `claude attach` window opens in the tmux session named for the git repo its working directory belongs to, so a job in a subdirectory or a worktree joins its repo's session rather than starting one of its own
**Status:** ready-for-agent
**blocked-by:** []

**Current behavior:**
`claude-sidebar attach` names the tmux session for the last component of the
session's `cwd`. A job started in `slurm-infra/roles` lands in a tmux session
`roles`, and one in `slurm-infra/.claude/worktrees/<name>` in a session of the
worktree's name, one per job, on the Mac and on mgmt-1.

**Desired behavior:**
- Where the `cwd` is inside a git repo, the tmux session is named for the
  main repo: the directory holding its common git directory, so a
  subdirectory and every linked worktree resolve to the same name
- Where the `cwd` is not inside a git repo, the name is its last component,
  as now
- The `.` and `:` rewriting tmux targets need applies as now
- On the Mac the name still matches the session sesh gives the same repo, so
  an attach window joins the repo's existing sesh session

The Maintainer chose the main repo's name over the cwd's last component.

**Key interfaces:**
- `claude-sidebar attach ID`: the tmux session it chooses from the session's
  `cwd`; runs on the session's own host, so the git lookup is local there
- `git -C "$cwd" rev-parse --path-format=absolute --git-common-dir`, whose
  parent is the main repo's root for a repo, a subdirectory and a worktree
  alike
- sesh's names for a directory: measure what sesh names a subdirectory and a
  worktree of a repo it lists (`dotfiles`) before relying on a match. If sesh
  names them differently and cannot be configured to agree, bring that back
  to the Maintainer rather than renaming sesh's sessions

**Acceptance criteria:**
- [ ] A Mac background session started in a subdirectory of `dotfiles`, and
      one in a `.claude/worktrees/<name>` worktree of it, each open in the
      `dotfiles` tmux session sesh uses
- [ ] A mgmt-1 background session in a subdirectory of a repo opens in the
      tmux session named for that repo, created if absent
- [x] A background session whose `cwd` is outside any git repo opens in a
      session named for the cwd's last component
- [ ] The Readme's paragraph on background entries says the session is
      named for the repo

**Out of scope:**
- Moving windows already open into their repo's session
- Interactive sessions started by hand, and how sesh names sessions it
  starts itself
