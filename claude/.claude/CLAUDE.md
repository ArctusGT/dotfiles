# Sub-agent models

`settings.json` sets `CLAUDE_CODE_SUBAGENT_MODEL=haiku`, so a sub-agent
started without a `model` runs on Haiku. Usage is the constraint: sub-agents
that inherited Opus were most of a day's spend.

- Leave `model` off (Haiku) for searches, probes, measurement runs and
  read-only reviews.
- Pass `model: "sonnet"` for `implement` runs and other long, exacting code
  work, where a Haiku redo would cost more than it saves.
- Never pass `"opus"` unless the Maintainer asks.
