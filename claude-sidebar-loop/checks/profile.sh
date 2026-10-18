#!/usr/bin/env bash
# Profile the CPU a running sidebar's loop and fzf take, as % of a core.
#   profile.sh FIXTURE [SECONDS]
# FIXTURE  still | working | waiting: 11 local sessions as state files, all
#          idle or, for the other two, one in that state, each in a pane of
#          a throwaway tmux server; no remote hosts
# SECONDS  how long to profile (default 60), after a 5 s warm-up
# SIDEBAR and BINARY choose what runs, as in lib.sh. The binary runs under a
# bash wrapper that, when stopped, prints `times` for it: its own CPU and
# every process it spawned and waited for (tmux), over the loop's whole life,
# its warm-up included; the first gather, which `claude-sidebar-loop rows`
# makes before the loop starts, is not. "binary" is its own CPU over the
# profiled window, from ps; "fzf" is fzf's. Prints the binary's path from its
# process. A throwaway tmux server, -L chkprof, and its own TMPDIR; changes
# nothing outside its temporary directory, and kills only its own server.
usage() { awk 'NR>1 && /^#/ {sub(/^# ?/, ""); print; next} NR>1 {exit}' "$0"; }
[[ $1 == --help ]] && { usage; exit 0; }
(( $# >= 1 )) || { usage >&2; exit 2; }
source "$(dirname "$0")/lib.sh"
fixture=$1 secs=${2:-60}
case $fixture in still | working | waiting) ;; *) echo "FIXTURE is still, working or waiting" >&2; exit 2 ;; esac
claim_server chkprof
scratch chkprof
cleanup() { tmux_on chkprof kill-server 2>/dev/null; kill $live 2>/dev/null; rm -rf "$run"; }
trap cleanup EXIT

tmux_on chkprof new-session -d -s work -x 50 -y 60 'sleep 3600'
for _ in $(seq 10); do tmux_on chkprof new-window -d -t work: 'sleep 3600'; done
mapfile -t panes < <(tmux_on chkprof list-panes -a -F '#{pane_id}')
for i in $(seq 0 10); do
  st=idle
  [[ $i == 3 && $fixture != still ]] && st=$fixture
  sid=$(printf '%08d-0000-4000-8000-%012d' "$i" "$i")
  state "$sid" "${panes[i]}" "$st" "/Users/x/repo$i"
  printf '{"name":"session %d","used_pct":%d}\n' "$i" $((i * 7)) >"$run/state/claude-sessions/$sid.context.json"
done

# the binary under a wrapper that reports its CPU when stopped
rm "$run/bin/claude-sidebar-loop"
cat >"$run/bin/claude-sidebar-loop" <<EOF
#!/usr/bin/env bash
[[ \$1 == rows ]] && exec "$BINARY" "\$@"
"$BINARY" "\$@" & pid=\$!
echo \$\$ \$EPOCHREALTIME >"$run/wrapper.pid"
trap 'kill \$pid; wait \$pid; times >"$run/times.new"; mv "$run/times.new" "$run/times"; exit' TERM
wait \$pid
EOF
chmod +x "$run/bin/claude-sidebar-loop"
tmux_on chkprof new-session -d -s side -x 50 -y 60 \
  "env PATH='$run/bin:$PATH' TMPDIR='$run/' HOME='$run/home' XDG_STATE_HOME='$run/state' SESH_CONFIG=/dev/null '$SIDEBAR'"

cpu() { # cumulative CPU in seconds of process $1, from ps's [[dd-]hh:]mm:ss.cc
  ps -o time= -p "$1" 2>/dev/null |
    perl -ne '@p = reverse split /[:-]/; printf "%.3f", $p[0] + 60 * $p[1] + 3600 * ($p[2] // 0)'
}
fzf_pid() { pgrep -f "fzf --read0.*$run" | head -1; }
bin_pid() { pgrep -f "^[^ ]*claude-sidebar-loop $run" | head -1; }

sleep 5
binary=$(lsof -a -p "$(bin_pid)" -d txt -Fn | sed -n 's/^n//p' | head -1)
f0=$(cpu "$(fzf_pid)") b0=$(cpu "$(bin_pid)")
sleep "$secs"
f1=$(cpu "$(fzf_pid)") b1=$(cpu "$(bin_pid)")
echo "$fixture, $secs s:"
read -r wrapper wrapper_start <"$run/wrapper.pid"
kill "$wrapper"
for _ in $(seq 50); do [[ -f $run/times ]] && break; sleep 0.1; done
life=$(perl -e 'printf "%.3f", $ARGV[1] - $ARGV[0]' "$wrapper_start" "$EPOCHREALTIME")
perl -e '($secs, $b0, $b1, $life, $binary, $file) = @ARGV; open F, $file; <F>; $_ = <F>;
  while (/(\d+)m([\d.]+)s/g) { $s += $1 * 60 + $2 }
  printf "  %-9s %5.1f%%  (own, last %d s, from ps: %s)\n", "binary", 100 * ($b1 - $b0) / $secs, $secs, $binary;
  printf "  %-9s %5.1f%%  (binary and its tmux, over its %.0f s life)\n", "loops", 100 * $s / $life, $life' \
  "$secs" "$b0" "$b1" "$life" "$binary" "$run/times"
perl -e 'printf "  %-9s %5.1f%%\n", "fzf", 100 * ($ARGV[2] - $ARGV[1]) / $ARGV[0]' "$secs" "$f0" "$f1"
