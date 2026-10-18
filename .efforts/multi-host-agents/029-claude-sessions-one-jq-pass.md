## claude-sessions reads every session in one jq pass

**Category:** enhancement
**Summary:** `claude-sessions` prints the same lines from a fixed number of spawns, not two `jq` and a `basename` for every session, so it costs a fraction of its ~67 ms of CPU a call on every host that runs it
**Status:** ready-for-closure
**blocked-by:**

**Current behavior:**
`claude-sessions` runs once a second on the Mac for the sidebar's local
entries, and once a second on each remote host over ssh for every Mac
sidebar polling it. For each state file it spawns a `basename`, a `jq`
to find the session's pid in Claude Code's records and a `jq` to merge
the record, context and subagent count, then one more `jq` over them
all. On the Mac, a process spawn costs about 2 ms of CPU, `jq` about
2.5 ms; with 7 sessions one call costs about 67 ms of CPU, and in the
live sidebar it was the largest single stage, 5.7% of a core of the
loops' 18.2%.

**Desired behavior:**
- The same output as today, byte for byte, for the same state files,
  records and panes, including which sessions are left out (a spare, an
  interactive client whose pane shows another session) and the pane each
  session is given
- Dead sessions' files are still removed as today: a session whose pid
  is gone, or missing from Claude Code's records where there are records
- The spawns no longer grow with the number of sessions: the per-session
  pid lookup and merge happen inside one `jq` run over every state file,
  context file and subagent count

The Maintainer chose to remove spawns in bash first, ahead of moving the
sidebar to Rust (030 to 032), from a measurement of the live sidebar
that put process spawns, not the jq render, at most of its cost.

**Key interfaces:**
- The output: one JSON object per live session, the shape the sidebar's
  `tag` and `format` and `claude-sidebar attach` read
- The inputs: the state directory (`<id>.json`, `<id>.context.json`,
  `<id>.agents/`), Claude Code's session records, and the default tmux
  server's panes
- `--help` keeps answering as today

**Acceptance criteria:**
- [x] On the Mac and on mgmt-1, today's and the new `claude-sessions`
      print identical output over the same inputs, by a script that runs
      both against copies of the state directory, including a dead
      session, a spare and a background job
- [x] A dead session's state, context and agents files are removed, as
      today
- [x] CPU a call measured before and after on the Mac with 7 or more
      sessions, by bash `time` over 50 calls, and stated in the closing
      commit with its method
- [x] The live sidebar's loops measured before and after by the stage
      profile (each stage under bash `time`, 60 s)

**Out of scope:**
- Moving `claude-sessions` or the sidebar to Rust (030 to 032)
- What `claude-sessions` reports or how it matches panes to sessions

**For the Maintainer:**
- A state or context file jq cannot parse now empties the whole call's
  output, where the old loop dropped only that session; an empty
  context file now shows its session with `context: null`, where the
  old loop dropped it. Both writers replace a file by `mv` only after
  jq succeeds, so neither file should arise; nothing guards for them.
  Accept, or ask for the final pass to read files that may not parse.
- The numbers in d0e8d48 were taken on 7ddeae0. Re-taken at e3e5593,
  same method: 24.04 ms CPU a call (from 65.70); live 60 s, sessions
  2.4% (from 6.4%), loops 15.4% (from 18.2%). The closing commit
  should carry these.
- mgmt-1 still runs the old script until its dotfiles clone is pulled.
- `bash -n` cannot check `claude-sessions` since db6253c (extglob).
- After closure review, 13e37f0 changed one rule on the Maintainer's
  ask: a pane tag naming no live session no longer decides the pane,
  so a job cleared with /clear while attached keeps its pane. Out of
  this slice's byte-for-byte scope by design; 032 ports the new rule.
