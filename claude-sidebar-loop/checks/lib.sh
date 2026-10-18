# Sourced by the checks, which each run a sidebar script and a claude-sidebar-loop
# binary in a throwaway tmux server and a directory of their own.
#   SIDEBAR  the claude-sidebar script (default: the repo's)
#   BINARY   the claude-sidebar-loop binary (default: this crate, built
#            into its target/ if absent)
# Nothing here touches the usual tmux, ~/.local or the TMPDIR a live sidebar
# keeps its cache in.

here=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
crate=$(dirname "$here")
repo=$(dirname "$crate")
SIDEBAR=$(realpath "${SIDEBAR:-$repo/bin/.local/bin/claude-sidebar}")
if [[ -z $BINARY ]]; then
  BINARY=$crate/target/release/claude-sidebar-loop
  [[ -x $BINARY ]] || cargo build --release --locked --quiet --manifest-path "$crate/Cargo.toml" || exit 1
fi
BINARY=$(realpath "$BINARY")
real_tmux=$(command -v tmux)

# a tmux server named $1, never the usual one; leaves it alone if running
tmux_on() { env -u TMUX -u TMUX_PANE "$real_tmux" -L "$@"; }
claim_server() { # $1 name
  tmux_on "$1" ls >/dev/null 2>&1 && { echo "tmux -L $1 already running; not touching it" >&2; exit 1; }
  return 0
}

# a directory for one run: short, as fzf's socket path must fit in 104
# bytes, with a `bin/` first on the PATH of everything the check runs, holding
# the binary and a `tmux` that reaches server $1 wherever TMUX is unset, as the
# usual tmux does for claude-sessions. HOME and the state directory are
# its own
scratch() { # $1 server the stub tmux reaches when TMUX is unset
  run=$(mktemp -d -t x)
  mkdir -p "$run/bin" "$run/home" "$run/state/claude-sessions"
  ln -s "$BINARY" "$run/bin/claude-sidebar-loop"
  cat >"$run/bin/tmux" <<STUB
#!/bin/sh
# the frame's tmux where TMUX is set, else the usual tmux
if [ -n "\$TMUX" ]; then exec "$real_tmux" "\$@"; fi
exec "$real_tmux" -L $1 "\$@"
STUB
  chmod +x "$run/bin/tmux"
  sleep 3600 &
  live=$!
}

# run a command as the sidebar runs: the scratch bin first on PATH, and its
# own TMPDIR (so its cache), HOME and state directory
in_scratch() {
  env PATH="$run/bin:$PATH" TMPDIR="$run/" HOME="$run/home" XDG_STATE_HOME="$run/state" \
    SESH_CONFIG="${SESH_CONFIG:-/dev/null}" "$@"
}

# the state file of a session whose process is $live
state() { # $1 id, $2 pane, $3 state, $4 cwd
  printf '{"session_id":"%s","pid":%d,"cwd":"%s","tmux_pane":"%s","entrypoint":"cli","state":"%s","since":%d}\n' \
    "$1" "$live" "${4:-/r/$1}" "$2" "$3" "$(date +%s)" >"$run/state/claude-sessions/$1.json.new"
  mv "$run/state/claude-sessions/$1.json.new" "$run/state/claude-sessions/$1.json"
}

fail=0
check() { # $1 description, $2.. test
  local what=$1; shift
  if "$@"; then echo "ok    $what"; else echo "FAIL  $what"; fail=1; fi
}
