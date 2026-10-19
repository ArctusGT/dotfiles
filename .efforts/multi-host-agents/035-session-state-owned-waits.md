## A session waits until the actor that asked moves on

**Category:** bug
**Summary:** `claude-session-state` records each wait against the actor it blocks, the main agent or one subagent, and derives the session's state by priority, waiting over working over idle, so a question left for the Maintainer shows as waiting whatever the other actors do, and a wait no event can resolve is never raised
**Status:** ready-for-agent
**blocked-by:**

**Current behavior:**
The state file holds one word and the last hook event wins. `resume`
runs on every `PostToolUse` from any actor, `stop` runs when the main
turn ends whatever the subagents are doing, and `wait` runs on a
`Notification`, which names no actor and no tool and which nothing
resolves. Replaying hook sequences through the script shows three
misreports, each seen live by the Maintainer on the sidebar:

- A question is up and a background subagent's tool completes: the
  subagent's `PostToolUse` turns waiting into working, and Clawd runs
  while the Maintainer is being asked something
- A subagent's permission prompt is up and the main turn ends: `Stop`
  turns it into idle, and idle with subagents running squats
- A prompt is resolved by a flow that runs no tool hooks, as the
  auto-mode setup's save did: its `Notification` set waiting and the
  session read waiting until its next prompt, long after the chat ended

A hooks log taken by the Maintainer settles what the docs leave open: a
subagent's tool events carry the parent's `session_id` and their own
`agent_id`; answering `AskUserQuestion` fires its `PostToolUse` with the
same `tool_use_id`, and `UserPromptSubmit` too; `AskUserQuestion` fires
`PermissionRequest`; the pending-prompt `Notification` arrives about six
seconds after the prompt; `Stop` fires after background subagents end.
No event fires when a prompt is denied by hand.

**Desired behavior:**
- A wait belongs to the actor the event fires in: the main agent when
  the hook input has no `agent_id`, else that subagent. It is raised by
  `PermissionRequest` and by `PreToolUse` for `AskUserQuestion`, each
  immediate and owned. `Notification` raises nothing.
- A wait clears only on its owner's own events: that actor's next
  `PostToolUse` or `PostToolUseFailure`, its `SubagentStop`, the main
  agent's `Stop`, or a `UserPromptSubmit`, which clears the main agent's.
  Another actor's events leave it standing.
- The state the file reports is derived, in this priority: waiting while
  any wait stands; else working while the main turn is open, from
  `UserPromptSubmit` until `Stop`; else idle. `since` moves when the
  derived state changes, as today.
- The readers are unchanged: `claude-sessions`, the loop's own reader
  and the renderer keep reading `state`, `since` and the subagent count
  as they do now, so a host whose script predates this still lists.
- The three sequences above replay to waiting, waiting, and working then
  idle. Every sequence the script handles today that involves one actor
  replays to the same words as before.
- The Maintainer accepted the trade: a dialog raised by a flow with no
  tool hooks, such as the auto-mode setup's save, no longer shows as
  waiting at all, in place of showing it forever.

**Key interfaces:**
- `claude-session-state`'s events and the state file it writes under
  the sessions state directory: the open waits and the main turn's
  openness are recorded there beside `state`, in whatever shape keeps
  the file one object readers pass through whole
- The `hooks` block of the stowed user settings, which maps Claude
  Code's events to the script's: `PermissionRequest` and
  `PostToolUseFailure` are new to it; the `Notification` entry that
  called the script goes, the one calling `claude-tmux-status` stays
- A replay check in the crate's checks, driven by a sequence of hook
  inputs on stdin against a scratch `XDG_STATE_HOME`, printing the
  state word after each event, so a sequence can be added when a new
  misreport is seen

**Acceptance criteria:**
- [ ] The replay check ships, answers `--help`, and runs the three
      sequences above and the one-actor sequences to the words stated,
      from a clean checkout on the Mac
- [ ] With the replay check, a wait raised by a subagent stands through
      the main agent's `PostToolUse` and `Stop`, and clears on that
      subagent's next `PostToolUse` or its `SubagentStop`
- [ ] `claude-sessions` and `claude-sidebar-loop sessions` still agree
      byte for byte over the shipped comparison, with state files the
      new script wrote
- [ ] The Maintainer runs the hooks experiment again, with a subagent
      command that does prompt, and sees the sidebar hold waiting while
      the question is up as the subagent completes, and sees waiting
      clear after denying the subagent's prompt once that subagent
      moves on or stops
- [ ] A session that ran the auto-mode setup's save reads idle in the
      sidebar once the chat ends, on the Mac and on mgmt-1 after stow

**Out of scope:**
- A Clawd pose or animation for any state; the renderer is unchanged
- `claude-tmux-status`, its `Notification` hook and `@claude_wait`
  (034 covers the sidebar's unseen mark)
- A timeout or sweep for a wait whose owner never moves on
- Showing which actor waits, or how many
