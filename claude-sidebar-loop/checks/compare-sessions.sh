#!/usr/bin/env bash
# Run claude-sessions and `claude-sidebar-loop sessions` over identical
# copies of one state directory and Claude Code's session records, and report
# whether their output and the files they leave behind are byte for byte the
# same.
#   compare-sessions.sh        (BINARY chooses the claude-sidebar-loop, as in lib.sh)
# The reference is the repo's claude-sessions. The state holds: a session with
# no record (dead), one whose record's pid is dead, a spare (record spare:
# true, live pid), a background job whose state file holds a dead daemon pid
# but whose record's pid is live, a tagged pane naming the job, an interactive
# client recorded in that pane (left out: the pane shows another session), a
# pane tagged with no live session, an untagged Claude pane titled with a
# session's name, a desktop app session, context percentages written 1e2,
# 42.50 and 0.1e1, a state file with an unknown field, and a non-JSON state
# file. Runs every case twice: with the records, and with no records (pids
# from the state files: live, dead, none). The panes are a file read through a
# stub tmux first on PATH, so they cannot change between runs. Works in a
# temporary directory, which it removes; changes nothing else, and kills only
# the `sleep` it starts for a live pid.
usage() { awk 'NR>1 && /^#/ {sub(/^# ?/, ""); print; next} NR>1 {exit}' "$0"; }
[[ $1 == --help ]] && { usage; exit 0; }
(( $# == 0 )) || { usage >&2; exit 2; }
source "$(dirname "$0")/lib.sh"
reference=$repo/bin/.local/bin/claude-sessions binary=$BINARY
work=$(mktemp -d -t x)
src_state=$work/src-state src_records=$work/src-records panes_file=$work/src-panes
mkdir -p "$src_state" "$src_records"
: >"$panes_file"
trap 'kill $live 2>/dev/null; rm -rf "$work"' EXIT
sleep 600 & live=$!
true & dead=$!; wait $dead

mkdir -p "$work/bin"
{ echo '#!/bin/sh'; echo "cat '$work/panes'"; } >"$work/bin/tmux"; chmod +x "$work/bin/tmux"
{ echo '#!/bin/sh'; echo "exec '$binary' sessions"; } >"$work/new"; chmod +x "$work/new"

fixture_state() { # $1 dir, $2 sid, $3 pid ("" for none), $4 pane ("" for none), $5 used_pct literal, $6 entrypoint
  jq -nc --arg s "$2" --arg p "$3" --arg pane "$4" --arg e "${6:-cli}" \
    '{session_id: $s, pid: (if $p == "" then null else ($p | tonumber) end), cwd: "/x",
      tmux_pane: (if $pane == "" then null else $pane end), entrypoint: $e, state: "idle", since: 1}' >"$1/$2.json"
  printf '{"name":"ctx-%s","used_pct":%s}\n' "$2" "${5:-1}" >"$1/$2.context.json"
  mkdir -p "$1/$2.agents"; : >"$1/$2.agents/a1"; : >"$1/$2.agents/a2"
}
record() { # $1 dir, $2 sid, $3 pid, $4 kind, $5 name, $6 spare, $7 jobId
  jq -nc --arg s "$2" --argjson p "$3" --arg k "$4" --arg n "$5" --argjson sp "$6" --arg j "$7" \
    '{sessionId: $s, pid: $p, kind: $k, name: $n, spare: $sp, jobId: (if $j == "" then null else $j end)}' >"$1/$3.json"
}

build() { # $1 fixture root, $2 with|without records
  local root=$1 st=$1/state/claude-sessions rec=$1/.claude/sessions
  mkdir -p "$st" "$rec"
  cp -R "$src_state/." "$st/"
  fixture_state "$st" fx-dead-norecord "$live" ""
  fixture_state "$st" fx-dead-recordpid "$live" ""
  fixture_state "$st" fx-spare "$live" ""
  fixture_state "$st" fx-job "$dead" "%fx1" 1e2
  fixture_state "$st" fx-nopid "" ""
  fixture_state "$st" fx-dead-pid "$dead" ""
  fixture_state "$st" fx-client "$live" "%fx1" 42.50
  fixture_state "$st" fx-titled "$live" "%fx9" 0.1e1
  fixture_state "$st" fx-desktop "$live" "" 7 claude-desktop
  jq -c '. + {extra: [1, 2.50]}' "$st/fx-titled.json" >"$st/fx-titled.json.new" && mv "$st/fx-titled.json.new" "$st/fx-titled.json"
  echo 'not json' >"$st/zz-broken.json"
  if [[ $2 == with ]]; then
    cp -R "$src_records/." "$rec/"
    record "$rec" fx-dead-recordpid "$dead" interactive named false ""
    record "$rec" fx-spare "$live" bg "" true ""
    jq -nc --arg s fx-job --argjson p "$live" \
      '{sessionId: $s, pid: $p, kind: "bg", name: "jobfx", spare: null, jobId: "jobfx"}' >"$rec/fx-job-$live.json"
    for sid in fx-client fx-titled fx-desktop; do
      record "$rec" "$sid" "$live" interactive "" null ""
      mv "$rec/$live.json" "$rec/$sid-$live.json"
    done
  fi
}

status=0
for mode in with without; do
  cp "$panes_file" "$work/panes"
  printf '%%fx1\tfx-job\tclaude\tjobfx\n' >>"$work/panes"
  printf '%%fx2\tgone-session\tclaude\t✳ nothing\n' >>"$work/panes"
  printf '%%fx3\t\tclaude\t◐ ctx-fx-titled\n' >>"$work/panes"
  for which in old new; do
    build "$work/$mode-$which" "$mode"
    script=$reference; [[ $which == new ]] && script=$work/new
    HOME=$work/$mode-$which XDG_STATE_HOME=$work/$mode-$which/state PATH=$work/bin:$PATH \
      "$script" </dev/null >"$work/$mode-$which.out" 2>"$work/$mode-$which.err"
    (cd "$work/$mode-$which/state" && find . | sort) >"$work/$mode-$which.tree"
  done
  for part in out err tree; do
    if cmp -s "$work/$mode-old.$part" "$work/$mode-new.$part"; then
      echo "$mode records, $part: identical ($(wc -l <"$work/$mode-old.$part" | tr -d ' ') lines)"
    else
      echo "$mode records, $part: DIFFER"; diff "$work/$mode-old.$part" "$work/$mode-new.$part"; status=1
    fi
  done
  echo "$mode records, fixture files left (claude-sessions):"
  grep 'fx-\|zz-' "$work/$mode-old.tree" | sed 's/^/  /'
  echo "$mode records, sessions printed (claude-sessions):"
  jq -r '"  \(.session_id) \(.tmux_pane) subagents=\(.subagents) name=\(.name) used_pct=\(.context.used_pct)"' "$work/$mode-old.out"
done
exit $status
