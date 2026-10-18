#!/usr/bin/env bash
# Check that a sidebar keeps working when another starts or goes beside it:
# `respawn-pane -k` on the sidebar ten times, each leaving a working sidebar,
# and a second sidebar started and stopped beside a running one, which keeps
# updating.
#   respawn.sh [N]    respawns (default 10); SIDEBAR and BINARY choose what
#                     runs, as in lib.sh; SETTLE the seconds waited after
#                     each respawn (default 1.5)
# A throwaway tmux server, -L chkrespawn, runs the sidebar in a 44 column
# pane, with a session whose state file the check rewrites. "Working" is the
# pane alive, one fzf of this run, and the rows in the cache a sidebar's
# pick and kick find changing with the state file within 5 s. TMPDIR is its
# own, so the cache is too. Changes nothing outside its temporary directory;
# kills only its own server.
[[ $1 == --help ]] && { awk 'NR>1 && /^#/ {sub(/^# ?/, ""); print; next} NR>1 {exit}' "$0"; exit 0; }
source "$(dirname "$0")/lib.sh"
n=${1:-10}
claim_server chkrespawn
scratch chkrespawn
cleanup() { tmux_on chkrespawn kill-server 2>/dev/null; kill $live 2>/dev/null; rm -rf "$run"; }
trap cleanup EXIT
cache=$run/claude-sidebar

pane=$(tmux_on chkrespawn new-session -d -P -F '#{pane_id}' -s work -x 120 -y 40 'sleep 3600')
id=aaaaaaaa-0000-4000-8000-000000000001
state $id "$pane" idle
# the sidebar's command in a pane: the scratch environment, then the script
start="env PATH='$run/bin:$PATH' TMPDIR='$run/' HOME='$run/home' XDG_STATE_HOME='$run/state' SESH_CONFIG=/dev/null '$SIDEBAR'"
sidebar() { # $1 the name of its own tmux session
  tmux_on chkrespawn new-session -d -P -F '#{pane_id}' -s "$1" -x 44 -y 30 "$start"
}

# whether the rows in the cache show state $1 within 5 s of writing it
shows() {
  state $id "$pane" "$1"
  for _ in $(seq 100); do
    grep -aq " $1 " "$cache/rows.0" 2>/dev/null && return 0
    sleep 0.05
  done
  return 1
}
# the fzf processes of this run
fzfs() { pgrep -f "fzf --read0" | while read -r p; do ps -o command= -p "$p"; done | grep -c "$run/"; }
panes_of() { tmux_on chkrespawn list-panes -t "$1" -F '#{pane_id}' 2>/dev/null | wc -l | tr -d ' '; }
is() { [[ $1 == "$2" ]]; }

states=(working waiting idle)
sidebar side >/dev/null
sleep 2
check "the sidebar starts and its rows update" shows working
for ((i = 1; i <= n; i++)); do
  tmux_on chkrespawn respawn-pane -k -t side "$start"
  sleep "${SETTLE:-1.5}"
  check "respawn $i: the pane lives, with one fzf" is "$(panes_of side) $(fzfs)" "1 1"
  check "respawn $i: rows update" shows "${states[i % 3]}"
done

# a second sidebar, on the same TMPDIR, started and stopped beside the first
sidebar second >/dev/null
sleep 2
tmux_on chkrespawn kill-session -t second
sleep 3
check "after a second sidebar came and went, the first's pane lives, with one fzf" is "$(panes_of side) $(fzfs)" "1 1"
check "and its rows still update" shows working
exit $fail
