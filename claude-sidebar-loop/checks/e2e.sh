#!/usr/bin/env bash
# Drive a throwaway sidebar end to end and check: a kick gathers at once;
# `pick N` jumps to entry N, local and remote, with no tagged file, and with
# no sidebar running; a jump made during a slow gather keeps fzf's cursor; an
# unreachable host shows within one ssh ConnectTimeout while the other hosts
# keep updating.
#   e2e.sh        (SIDEBAR and BINARY choose what runs, as in lib.sh)
# Two throwaway tmux servers: -L chke2e is the frame (a right pane attached
# to the usual tmux, and the sidebar), -L chke2eu the usual tmux, reached
# through a tmux wrapper on PATH whenever TMUX is unset. Local sessions are
# state files under its own XDG_STATE_HOME and HOME; hosts come from its own
# sesh config and a stub ssh: `fake` answers two sessions at once, `slow`
# answers after 2 s, `down` fails after 3 s, as a host past ConnectTimeout.
# TMPDIR is its own, so the sidebar's cache is too. Takes about a minute.
# Changes nothing outside its temporary directory; kills only its own two
# servers and its own sleep.
[[ $1 == --help || $# -ne 0 ]] && { awk 'NR>1 && /^#/ {sub(/^# ?/, ""); print; next} NR>1 {exit}' "$0"; exit 0; }
source "$(dirname "$0")/lib.sh"
f() { tmux_on chke2e "$@"; }
w() { tmux_on chke2eu "$@"; }
claim_server chke2e
claim_server chke2eu
scratch chke2eu
cleanup() {
  f ls >/dev/null 2>&1 && f kill-server
  w ls >/dev/null 2>&1 && w kill-server
  kill $live 2>/dev/null
  rm -rf "$run"
}
trap cleanup EXIT
cache=$run/claude-sidebar
mkdir -p "$run/nosidebar"
cat >"$run/bin/ssh" <<EOF
#!/bin/sh
for a; do host=\$prev; prev=\$a; done
case "\$prev" in
  claude-sessions*) ;;
  *) echo "\$host \$prev" >>"$run/ssh-jumps"; exit 0 ;;
esac
case "\$host" in
  fake) cat "$run/fake.out" ;;
  slow) sleep 2; exit 0 ;;
  down) sleep 3; exit 255 ;;
esac
EOF
chmod +x "$run/bin/ssh"
now=$(date +%s)
printf '{"session_id":"00000000-0000-4000-8000-0000000000f1","pid":1,"cwd":"/r/remote1","tmux_pane":"%%71","entrypoint":"cli","state":"idle","since":%d,"name":"remote one","subagents":0,"context":null}\n{"session_id":"00000000-0000-4000-8000-0000000000f2","pid":1,"cwd":"/r/remote2","tmux_pane":"%%72","entrypoint":"cli","state":"idle","since":%d,"name":"remote two","subagents":0,"context":null}\n\036%%71\n' "$now" "$now" >"$run/fake.out"
SESH_CONFIG=$run/sesh.toml
sesh() { # the hosts, by name
  : >"$run/sesh.toml"
  for h; do printf '[[session]]\nname = "%s"\nstartup_command = "ssh-tmux %s"\n\n' "$h" "$h" >>"$run/sesh.toml"; done
}
sesh fake

# the usual tmux: two local panes, and fake's relay window
pane_a=$(w new-session -d -P -F '#{pane_id}' -s work -x 120 -y 40 'sleep 3600')
pane_b=$(w split-window -P -F '#{pane_id}' -t "$pane_a" 'sleep 3600')
relay_window=$(w new-window -d -P -F '#{window_id}' -t work: 'sleep 3600')
w set -w -t "$relay_window" @ssh_tmux fake
state aaaaaaaa-0000-4000-8000-000000000001 "$pane_a" idle
state bbbbbbbb-0000-4000-8000-000000000002 "$pane_b" idle
w select-pane -t "$pane_a"

start="env PATH='$run/bin:$PATH' TMPDIR='$run/' HOME='$run/home' XDG_STATE_HOME='$run/state' SESH_CONFIG='$run/sesh.toml' '$SIDEBAR'"
# the frame: its right pane a client of the usual tmux, on pane a
f new-session -d -s frame -x 160 -y 50 "env -u TMUX -u TMUX_PANE $real_tmux -L chke2eu attach -t work"
f set -p @frame_main 1
f split-window -hbf -l 44 "$start"
frame_socket=$(f display -p '#{socket_path}')
client() { w list-clients -F '#{pane_id} #{window_id}' | head -1; }
# the sidebar's commands, as the frame's tmux hooks and keys run them
cmd() { in_scratch env TMUX="$frame_socket,0,0" "$SIDEBAR" "$@"; }
cursor() { # fzf's current item, from 1
  curl -sf --unix-socket "$cache/fzf.sock" http://localhost | jq '.current.index + 1' 2>/dev/null
}
mtime() { stat -f %Fm "$cache/rows.0" 2>/dev/null; }
wait_rows() { # $1 perl regex rows.0 must match, $2 seconds; prints the time it took
  local t0=$EPOCHREALTIME
  while ! perl -0777 -ne "exit !(/$1/)" "$cache/rows.0" 2>/dev/null; do
    (( ${EPOCHREALTIME/./} - ${t0/./} > $2 * 1000000 )) && { echo timeout; return 1; }
    sleep 0.02
  done
  echo "$(( (${EPOCHREALTIME/./} - ${t0/./}) / 1000 )) ms"
}

wait_rows 'remote two' 10 >/dev/null
sleep 1.5
items=$(perl -0ne 'chomp; @f = split /\t/, $_, -1; $f[6] =~ s/\e\[[0-9;]*m//g; ($t) = $f[6] =~ /(\S.*)/; print "[$f[0]|$f[1]|$f[2]|$f[4]|$f[5]] $t\n"' "$cache/rows.0")
echo "rows.0:"; echo "$items" | sed 's/^/  /'
check "four numbered entries, local first" test "$(echo "$items" | awk -F'|' '$4 != "" {printf "%s ", $4}')" = "1 2 3 4 "
check "the right pane's entry on show" grep -q "^\[|$pane_a||1|1\]" <<<"$items"
check "the cursor follows it to item 2" test "$(cursor)" = 2
check "no tagged file in the cache" test ! -e "$cache/tagged"

# a kick, just after a gather: the change shows at once, not at the next second
m=$(mtime); while [[ $(mtime) == "$m" ]]; do sleep 0.01; done
state aaaaaaaa-0000-4000-8000-000000000001 "$pane_a" working
cmd kick
took=$(wait_rows 'working' 2)
echo "      kick to rows: $took"
check "a kick gathers at once (under 500 ms)" test "${took% ms}" -lt 500 2>/dev/null

# pick N: local, then remote
cmd pick 2
sleep 0.5
check "pick 2 moves the right pane's client to pane b" test "$(client | cut -d' ' -f1)" = "$pane_b"
check "and fzf's cursor to its item, 3" test "$(cursor)" = 3
cmd pick 3
sleep 1
check "pick 3 moves the client to fake's relay window" test "$(client | cut -d' ' -f2)" = "$relay_window"
check "and asks fake to show its pane %71" grep -q "pane='%71'" "$run/ssh-jumps"
check "and fzf's cursor to item 5" test "$(cursor)" = 5
check "still no tagged file" test ! -e "$cache/tagged"

# with no sidebar running: pick gathers once for itself
env TMUX="$frame_socket,0,0" PATH="$run/bin:$PATH" TMPDIR="$run/nosidebar/" HOME="$run/home" \
  XDG_STATE_HOME="$run/state" SESH_CONFIG="$run/sesh.toml" "$SIDEBAR" pick 1
sleep 0.5
check "pick 1 with no sidebar moves the client to pane a" test "$(client | cut -d' ' -f1)" = "$pane_a"

# a jump during a slow gather: the gather began with pane a on show and
# ends after the jump to pane b; the cursor stays on b's item
sesh fake slow
m=$(mtime); while [[ $(mtime) == "$m" ]]; do sleep 0.01; done  # a gather done
sleep 1.5                                                     # the next one under way, held by slow
w switch-client -t "$pane_a"; w select-pane -t "$pane_a"
cmd kick
sleep 3                                                       # rows show pane a, cursor on 2
before=$(cursor)
m=$(mtime); while [[ $(mtime) == "$m" ]]; do sleep 0.01; done
sleep 2                                                       # mid-gather again
cmd pick 2
positions=""
for _ in $(seq 40); do positions+="$(cursor) "; sleep 0.1; done
echo "      cursor before $before, then every 100 ms: $positions"
check "a jump during a gather keeps the cursor on its entry" test -z "$(tr ' ' '\n' <<<"$positions" | grep -vx 3 | grep .)"

# an unreachable host: shown within one ConnectTimeout (3 s) of its first
# gather, while the other hosts and this one keep updating
sesh fake down
m=$(mtime); while [[ $(mtime) == "$m" ]]; do sleep 0.01; done
cmd kick
took=$(wait_rows 'unreachable' 6)
echo "      kick to unreachable shown: $took"
check "unreachable within one ConnectTimeout (under 3.5 s)" test "${took% ms}" -lt 3500 2>/dev/null
state bbbbbbbb-0000-4000-8000-000000000002 "$pane_b" waiting
took=$(wait_rows 'waiting' 9)
echo "      local change shown: $took"
check "this host still updates, within two gathers of 3 s and a second" test "${took% ms}" -lt 8000 2>/dev/null
sed -i '' 's/remote two/remote 2b/' "$run/fake.out"
took=$(wait_rows 'remote 2b' 9)
echo "      fake's change shown: $took"
check "the other host still updates" test "${took% ms}" -lt 8000 2>/dev/null
echo "log:"; sed 's/^/  /' "$run/claude-sidebar.log" 2>/dev/null
exit $fail
