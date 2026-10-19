#!/usr/bin/env bash
# Check the sidebar keeps its mouse through a jump. First fzf alone, as the
# record of why the workaround exists: posts of the sidebar's own actions
# during a silent execute of a second or more turn fzf's mouse off for good
# once it returns (fzf 0.74.5), where the same posts while idle do not. Then
# a throwaway sidebar with a working session's Clawd moving: N click jumps
# between two entries, each switch held 1.2 s, as a jump that opens a
# background session takes, and the pane's mouse flags read after each.
#   mouse.sh [N]    N click jumps (default 30); SIDEBAR and BINARY choose
#                   what runs, as in lib.sh
# Three throwaway tmux servers: -L chkmousef for fzf alone, -L chkmouse the
# frame (a right pane attached to the usual tmux, and the sidebar), and
# -L chkmouseu the usual tmux. Clicks are mouse bytes in SGR encoding (the
# `\e[<` reports fzf asks for), sent to the sidebar's pane. Changes nothing
# outside its temporary directory; kills only its own servers and its own
# sleep.
usage() { awk 'NR>1 && /^#/ {sub(/^# ?/, ""); print; next} NR>1 {exit}' "$0"; }
[[ $1 == --help ]] && { usage; exit 0; }
jumps=${1:-30}
[[ $# -le 1 && $jumps =~ ^[0-9]+$ ]] || { usage >&2; exit 2; }
source "$(dirname "$0")/lib.sh"
a() { tmux_on chkmousef "$@"; }
f() { tmux_on chkmouse "$@"; }
w() { tmux_on chkmouseu "$@"; }
claim_server chkmousef
claim_server chkmouse
claim_server chkmouseu
scratch chkmouseu
cleanup() {
  for s in a f w; do $s ls >/dev/null 2>&1 && $s kill-server; done
  kill $live 2>/dev/null
  rm -rf "$run"
}
trap cleanup EXIT
trap 'exit 1' TERM INT

# a pane's button-event, any-event and SGR mouse flags, as fzf turns them on
mouse() { # $1 tmux server name, $2 pane
  tmux_on "$1" display -p -t "$2" '#{mouse_button_flag}#{mouse_any_flag}#{mouse_sgr_flag}'
}
until_socket() { # $1 socket
  for _ in $(seq 50); do [[ -S $1 ]] && return 0; sleep 0.1; done
  return 1
}

# fzf alone, its enter a 2 s silent execute
mkdir -p "$run/alone"
printf 'a\0b\0c\0' >"$run/alone/rows"
sock=$run/alone/fzf.sock
alone=$(a new-session -d -P -F '#{pane_id}' -x 80 -y 20 \
  "fzf --read0 --listen '$sock' --bind 'enter:execute-silent(sleep 2)' <'$run/alone/rows'")
until_socket "$sock"
sleep 0.3
post() { curl -sf --unix-socket "$sock" -X POST http://localhost -d "$1"; }
posts() { for _ in 1 2 3; do post "reload-sync(cat '$run/alone/rows')"; post 'pos(2)'; sleep 0.2; done; }
started=$(mouse chkmousef "$alone")
posts
check "fzf alone keeps its mouse under posts while idle" test "$(mouse chkmousef "$alone")" = "$started"
a send-keys -t "$alone" Enter
sleep 0.2
posts
sleep 2
echo "      fzf $(fzf --version | cut -d' ' -f1): flags $started at start, $(mouse chkmousef "$alone") after posts during an execute"
check "fzf alone drops its mouse under posts during an execute (the workaround's reason)" \
  test "$(mouse chkmousef "$alone")" = 000

# the sidebar: a's Clawd moving, b's still, each in a window of its own
pane_a=$(w new-session -d -P -F '#{pane_id}' -s work -x 120 -y 40 'sleep 3600')
pane_b=$(w new-window -d -P -F '#{pane_id}' -t work: 'sleep 3600')
state aaaaaaaa-0000-4000-8000-000000000001 "$pane_a" working
state bbbbbbbb-0000-4000-8000-000000000002 "$pane_b" idle
# a switch held past the second fzf needs to drop its mouse, as a jump that
# opens a background session takes, so frames come due while a jump runs
mv "$run/bin/tmux" "$run/bin/tmux.fast"
printf '#!/bin/sh\n[ "$1" = switch-client ] && sleep 1.2\nexec "%s" "$@"\n' "$run/bin/tmux.fast" >"$run/bin/tmux"
chmod +x "$run/bin/tmux"
start="env PATH='$run/bin:$PATH' TMPDIR='$run/' HOME='$run/home' XDG_STATE_HOME='$run/state' SESH_CONFIG=/dev/null '$SIDEBAR'"
f new-session -d -s frame -x 160 -y 50 "env -u TMUX -u TMUX_PANE $real_tmux -L chkmouseu attach -t work"
f set -p @frame_main 1
side=$(f split-window -hbf -l 44 -P -F '#{pane_id}' "$start")
cache=$run/claude-sidebar
until_socket "$cache/fzf.sock"
sleep 2
client_pane() { w list-clients -F '#{pane_id}' | head -1; }
# the screen line, from 1, of the entry whose name starts $1
line_of() { f capture-pane -p -t "$side" | grep -n -m1 " $1-" | cut -d: -f1; }
click() { # $1 screen line, from 1: a left press and release, SGR encoded
  f send-keys -t "$side" -l $'\e[<0;10;'"$1"$'M\e[<0;10;'"$1"m
}
line_a=$(line_of aaaaaaaa)
line_b=$(line_of bbbbbbbb)
check "both entries on screen" test -n "$line_a" -a -n "$line_b"
# turn a dropped mouse back on, as fzf has it, so each jump counts alone
revive() { printf '\e[?1000h\e[?1002h\e[?1006h' >"$(f display -p -t "$side" '#{pane_tty}')"; }
dropped=0 missed=0
for k in $(seq "$jumps"); do
  if (( k % 2 )); then line=$line_b want=$pane_b; else line=$line_a want=$pane_a; fi
  click "$line"
  sleep 1.6
  if [[ $(mouse chkmouse "$side") != 111 ]]; then
    dropped=$((dropped + 1))
    revive
  fi
  [[ $(client_pane) == "$want" ]] || missed=$((missed + 1))
done
echo "      $jumps click jumps: mouse dropped after $dropped, client missed $missed"
check "the sidebar keeps its mouse through every click jump" test "$dropped" = 0
check "every click jumped" test "$missed" = 0
echo "log:"; sed 's/^/  /' "$run/claude-sidebar.log" 2>/dev/null
exit $fail
