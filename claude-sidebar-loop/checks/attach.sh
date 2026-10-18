#!/usr/bin/env bash
# Check that `claude-sidebar attach ID` finds or opens a session's pane without
# running claude-sessions: it prints the pane showing a session, and with
# none, opens `claude attach` in a new tagged window and finds that again.
#   attach.sh          (SIDEBAR and BINARY choose what runs, as in lib.sh)
# A throwaway tmux server, -L chkattach, stands for the usual tmux; a stub
# `claude-sessions` on the PATH records any run of it, and a stub `claude`
# stands for the CLI. Changes nothing outside its temporary directory; kills
# only its own server.
[[ $1 == --help ]] && { awk 'NR>1 && /^#/ {sub(/^# ?/, ""); print; next} NR>1 {exit}' "$0"; exit 0; }
source "$(dirname "$0")/lib.sh"
claim_server chkattach
scratch chkattach
cleanup() { tmux_on chkattach kill-server 2>/dev/null; kill $live 2>/dev/null; rm -rf "$run"; }
trap cleanup EXIT

printf '#!/bin/sh\necho spawned >>"%s/spawns"\n' "$run" >"$run/bin/claude-sessions"
cat >"$run/bin/claude" <<STUB
#!/bin/sh
case "\$1" in
  agents) echo '[{"sessionId":"bbbbbbbb-0000-4000-8000-000000000002","id":"job42"}]' ;;
  attach) echo "\$2" >"$run/attached"; exec sleep 3600 ;;
esac
STUB
chmod +x "$run/bin/claude-sessions" "$run/bin/claude"

pane=$(tmux_on chkattach new-session -d -P -F '#{pane_id}' -s work -x 120 -y 40 'sleep 3600')
mkdir -p "$run/repo"
state aaaaaaaa-0000-4000-8000-000000000001 "$pane" idle
state bbbbbbbb-0000-4000-8000-000000000002 "" idle "$run/repo"

attach() { in_scratch "$SIDEBAR" attach "$1"; }
check "a session with a pane prints it" test "$(attach aaaaaaaa-0000-4000-8000-000000000001)" = "$pane"
opened=$(attach bbbbbbbb-0000-4000-8000-000000000002)
sleep 0.5
check "a session with none opens a pane, tagged" test "$(tmux_on chkattach display -p -t "$opened" '#{@claude_attach}')" = bbbbbbbb-0000-4000-8000-000000000002
check "running claude attach on its job" test "$(cat "$run/attached" 2>/dev/null)" = job42
check "a second attach finds that pane" test "$(attach bbbbbbbb-0000-4000-8000-000000000002)" = "$opened"
check "the second opened nothing" test "$(tmux_on chkattach list-panes -a | wc -l | tr -d ' ')" = 2
check "claude-sessions was never run" test ! -e "$run/spawns"
exit $fail
