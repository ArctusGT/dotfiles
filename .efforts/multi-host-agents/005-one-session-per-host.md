## One Mac session per remote host

**Category:** bug
**Summary:** Reach each remote host through one Mac tmux session holding one remote client, so workmux's jumps move the client being looked at
**Status:** ready-for-agent
**blocked-by:** none

**Current behavior:**
The sesh picker lists one entry per remote repo (`slurm-infra@mgmt-1`,
`iot-infra@mgmt-1`). Each is its own Mac tmux session running `ssh-tmux`, so
each holds its own ssh connection and its own client on the remote tmux
server, pinned to the remote session named after the repo.

workmux on the remote jumps with `switch-client -t <target>` and no `-c`, so
the remote tmux picks the client: the most recently active one attached to the
session the command came from. A jump made from a window in one remote session
(a sidebar click, `workmux add`'s automatic switch) can move a client that
belongs to a different Mac session. That Mac session then shows the wrong
remote session, and switching to it on the Mac appears to do nothing. Recovery
today is killing the Mac session and reconnecting, or a hand-run
`switch-client -c`.

**Desired behavior:**
Each remote host has one entry in the sesh picker and one Mac session, holding
one ssh connection and one client on that host's tmux server. Moving between
the host's repos happens on the remote: workmux's sidebar and jumps, the
remote tmux's session chooser, or a remote picker. Whatever workmux switches is
then the client on screen.

Opening the host's entry from the picker reattaches to the remote session last
used, and creates a sensible default session on a first connect. Reconnect on
connection loss, the cross-host Claude marker (`@ssh_tmux` on the Mac window
plus the remote bell) and mouse copy keep working.

**Key interfaces:**
- The sesh config's remote entries: one per host instead of one per repo
- `ssh-tmux HOST SESSION [DIR]`: whether its SESSION argument still makes
  sense, or the host entry attaches to the remote server's most recent
  session (`tmux attach`, falling back to creating one)
- How a remote repo session gets created now that the Mac picker no longer
  does it: `workmux add`, the remote session chooser, or a picker on the
  remote (sesh is not installed on mgmt-1)
- Remote sessions that exist today (`slurm-infra`, `iot-infra`) carry over

**Acceptance criteria:**
- [x] The sesh picker shows one `mgmt-1` entry and no per-repo mgmt-1 entries
- [x] Opening it gives one Mac session whose window holds one client on
      mgmt-1's tmux server (`tmux list-clients` on mgmt-1 shows one client from
      the Mac)
- [x] From that session, the Maintainer can reach both `slurm-infra` and
      `iot-infra` on mgmt-1 without leaving the Mac session, and a workmux
      sidebar jump between them moves the view on screen
- [x] Closing and reopening the `mgmt-1` entry returns to the remote session
      last used
- [x] Dropping the connection reconnects as before, and the Claude marker for
      a waiting mgmt-1 agent still shows on the Mac
- [x] The Readme and the sesh config's comments describe the per-host entry

**Out of scope:**
- Changing workmux itself, such as passing `-c` to its `switch-client` calls
- The combined cross-host view (slice 004), which builds on the result
- Hosts other than mgmt-1
