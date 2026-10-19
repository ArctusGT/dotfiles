#!/usr/bin/env python3
"""Compare the rows two revisions of claude-sidebar-loop render, byte for byte,
over the same sessions.

  compare-rows.py [REV]
REV  the git revision whose crate is the reference (default HEAD); the
     working tree's crate is the one checked

Both are built (cargo, release, into their own temporary target directories),
then each runs in a throwaway tmux server (-L chkrows), in a pane 2 columns
wider than the rows, with a copy of the same fixture: local sessions in
every state (state files with context and subagents, one a background job
with no pane, one a desktop app session with none), a stub tmux (those panes,
and a frame whose right pane shows one of two things) and a stub ssh
answering for three sesh hosts: mgmt
(sessions in every state, numbers written 1e2, 42.50 and 2.0, a desktop app
session with and without a pane), lab (ssh fails: unreachable) and empty (no
sessions). Compares all 48 rows.N the loop writes and the `rows` it prints,
for widths 48 and 78, warn levels 50 and 30, and the frame's right pane on a
local pane and on mgmt's relay.

Changes nothing outside its temporary directory, and kills only its own tmux
server, which must not be running when it starts. With KEEP set, leaves its
temporary directory for a look, and says where.
"""
import argparse, atexit, json, os, re, shutil, subprocess, tempfile, time

ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
ap.add_argument("rev", nargs="?", default="HEAD")
args = ap.parse_args()
CRATE = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
REPO = os.path.dirname(CRATE)
SERVER = "chkrows"
# short, as a fixture root's paths stay clear of any socket limit
tmp = tempfile.mkdtemp(prefix="cr", dir="/tmp")
if not os.environ.get("KEEP"):
    atexit.register(shutil.rmtree, tmp, True)


def tmux(*a, check=True):
    env = {k: v for k, v in os.environ.items() if k not in ("TMUX", "TMUX_PANE")}
    return subprocess.run(["tmux", "-L", SERVER] + list(a), env=env, capture_output=True, text=True, check=check)


def build(crate_dir, target):
    subprocess.run(["cargo", "build", "--release", "--locked", "--quiet", "--manifest-path",
                    os.path.join(crate_dir, "Cargo.toml"), "--target-dir", target], check=True)
    return os.path.join(target, "release", "claude-sidebar-loop")


if tmux("ls", check=False).returncode == 0:
    raise SystemExit("tmux -L %s already running; not touching it" % SERVER)

reference_src = os.path.join(tmp, "reference-src")
os.makedirs(reference_src)
archive = subprocess.run(["git", "-C", REPO, "archive", args.rev, "claude-sidebar-loop"],
                         capture_output=True, check=True).stdout
subprocess.run(["tar", "-x", "-C", reference_src], input=archive, check=True)
old_binary = build(os.path.join(reference_src, "claude-sidebar-loop"), os.path.join(tmp, "target-old"))
new_binary = build(CRATE, os.path.join(tmp, "target-new"))

now = int(time.time())
live = os.getpid()


def remote(sid, state, ago, pane, name, sub, ctx, entry="cli"):
    return {"session_id": sid, "pid": 1, "cwd": "/home/sysadmin/r%s" % sid[-2:], "tmux_pane": pane,
            "entrypoint": entry, "state": state, "since": now - ago, "name": name, "subagents": sub,
            "context": ctx}


mgmt = [remote("00000000-0000-4000-8000-0000000000%02d" % i, *c) for i, c in enumerate([
    ("idle", 30, "%7", "idle", 0, {"used_pct": 0}),
    ("working", 90, "%8", "working", 0, {"used_pct": "@1e2"}),
    ("waiting", 1830, "%9", "waiting", 0, {"used_pct": "@42.50"}),
    ("idle", 3570, "%10", "idle subagents", "@2.0", {"used_pct": 50}),
    ("waiting", 3630, None, "background waiting", 3, {"used_pct": 51}),
    ("working", 7300, "%11", None, 1, None),
    ("idle", 90, None, "desktop no pane", 0, None, "claude-desktop"),
    ("idle", 90, "%12", "desktop pane", 0, {"name": None, "used_pct": None}, "claude-desktop"),
])]

# local sessions: id, state, seconds ago, pane, cwd, context file (None for
# no file), subagents running, entrypoint
LOCAL = [
    ("a1111111-0000-4000-8000-000000000001", "idle", 20, "%1", "/Users/x/one", {"name": "one", "used_pct": 7}, 0, "cli"),
    ("a2222222-0000-4000-8000-000000000002", "working", 400, "%2", "/Users/x/two", {"name": "two", "used_pct": 42.5}, 0, "cli"),
    ("a3333333-0000-4000-8000-000000000003", "waiting", 5000, "%3", "/Users/x/three", None, 2, "cli"),
    ("a4444444-0000-4000-8000-000000000004", "idle", 90, "%4", "/Users/x/four", {"used_pct": 80}, 1, "cli"),
    ("a5555555-0000-4000-8000-000000000005", "working", 90, None, "/Users/x/five", {"used_pct": 12}, 0, "cli"),
    ("a6666666-0000-4000-8000-000000000006", "idle", 90, None, "/Users/x/six", None, 0, "claude-desktop"),
]
# the default tmux server's panes: id, tag, command, @claude_wait, title; a
# revision from before the wait column is served the line without it
PANES = [("%1", "", "claude", "", "one"), ("%2", "", "claude", "", "two"), ("%3", "", "claude", "", "three"),
         ("%4", "", "claude", "", "four"), ("%5", "", "zsh", "", "arc@host:~")]


def fixture(shown):
    """a fixture root for one side: state, stub bin, sesh config"""
    root = tempfile.mkdtemp(dir=tmp)
    state = os.path.join(root, "state/claude-sessions")
    os.makedirs(state)
    os.makedirs(os.path.join(root, "home/.claude/sessions"))
    os.makedirs(os.path.join(root, "bin"))
    for sid, st, ago, pane, cwd, ctx, running, entry in LOCAL:
        with open(os.path.join(state, sid + ".json"), "w") as f:
            json.dump({"session_id": sid, "pid": live, "cwd": cwd, "tmux_pane": pane, "entrypoint": entry,
                       "state": st, "since": now - ago}, f)
        if ctx is not None:
            with open(os.path.join(state, sid + ".context.json"), "w") as f:
                json.dump(ctx, f)
        os.makedirs(os.path.join(state, sid + ".agents"))
        for k in range(running):
            open(os.path.join(state, sid + ".agents", "agent%d" % k), "w").close()
    with open(os.path.join(root, "panes"), "w") as f:
        f.write("".join("\t".join(p) + "\n" for p in PANES))
    lines = "".join(json.dumps(s, separators=(",", ":")) + "\n" for s in mgmt)
    # numbers written as given: "@1e2" becomes the bare literal 1e2
    lines = re.sub(r'"@([^"]*)"', r"\1", lines)
    open(os.path.join(root, "mgmt.out"), "w").write(lines + "\036%8\n")
    open(os.path.join(root, "empty.out"), "w").write("\036\n")
    open(os.path.join(root, "shown"), "w").write(shown + "\n")
    stub_tmux = """#!/bin/sh
case "$1 $2" in
  "list-panes -a") case "$*" in
    *claude_wait*) cat '%(r)s/panes' ;;
    *) awk -F'\t' -v OFS='\t' '{ print $1, $2, $3, $5 }' '%(r)s/panes' ;;
  esac ;;
  "list-panes -s") echo '1 %%frame /dev/fake' ;;
  "display -p") cat '%(r)s/shown' ;;
esac
""" % {"r": root}
    stub_ssh = """#!/bin/sh
for a; do host=$prev; prev=$a; done
case "$host" in
  sysadmin@mgmt) cat '%(r)s/mgmt.out' ;;
  empty-host) cat '%(r)s/empty.out' ;;
  *) exit 255 ;;
esac
""" % {"r": root}
    for name, text in (("tmux", stub_tmux), ("ssh", stub_ssh)):
        p = os.path.join(root, "bin", name)
        open(p, "w").write(text)
        os.chmod(p, 0o755)
    open(os.path.join(root, "sesh.toml"), "w").write(
        '[[session]]\nname = "mgmt"\nstartup_command = "ssh-tmux sysadmin@mgmt"\n\n'
        '[[session]]\nname = "lab"\nstartup_command = "ssh-tmux lab-host"\n\n'
        '[[session]]\nname = "empty"\nstartup_command = "ssh-tmux empty-host"\n')
    cache = os.path.join(root, "cache")
    os.mkdir(cache)
    open(os.path.join(cache, "jumped"), "w").write("j\n")
    os.mkfifo(os.path.join(cache, "kick"))
    env = "PATH=%s/bin:$PATH HOME=%s/home XDG_STATE_HOME=%s/state SESH_CONFIG=%s/sesh.toml TMPDIR=%s/" % (
        root, root, root, root, root)
    return cache, env


def wait_for(path, secs=15):
    end = time.time() + secs
    while not os.path.exists(path):
        if time.time() > end:
            raise SystemExit("timed out waiting for %s" % path)
        time.sleep(0.05)


def run(binary, shown, width, warn, name):
    """the 48 frames the loop writes, and the rows it prints, for this binary"""
    cache, env = fixture(shown)
    warn_env = "-u CLAUDE_CTX_WARN" if warn is None else "CLAUDE_CTX_WARN=%s" % warn
    tmux("new-session", "-d", "-s", name, "-x", str(width + 2), "-y", "50",
         "env %s %s %s rows >%s/rows.once; touch %s/once.done; exec env %s %s %s %s" % (
             warn_env, env, binary, cache, cache, warn_env, env, binary, cache))
    wait_for(os.path.join(cache, "once.done"))
    wait_for(os.path.join(cache, "rows.47"))
    time.sleep(0.3)
    out = ([open(os.path.join(cache, "rows.%d" % k), "rb").read() for k in range(48)],
           open(os.path.join(cache, "rows.once"), "rb").read())
    tmux("kill-session", "-t", name)
    return out


failures = 0
n = 0
try:
    for shown_name, shown in (("local %2", "%2 "), ("mgmt relay", "%5 sysadmin@mgmt")):
        for width in (48, 78):
            for warn in (None, 30):
                for attempt in range(2):
                    n += 1
                    old_frames, old_once = run(old_binary, shown, width, warn, "o%d" % n)
                    new_frames, new_once = run(new_binary, shown, width, warn, "n%d" % n)
                    same = sum(a == b for a, b in zip(old_frames, new_frames))
                    ok = same == 48 and old_once == new_once
                    if ok:
                        break
                failures += not ok
                print("shown %-10s width %2d warn %-3s  frames equal %2d/48  rows equal %-5s  items %2d  %s"
                      % (shown_name, width, warn or "50", same, old_once == new_once,
                         old_frames[0].count(b"\0"), "ok" if ok else "DIFFER"))
                if not ok:
                    k = next((k for k in range(48) if old_frames[k] != new_frames[k]), 0)
                    a, b = (old_frames[k], new_frames[k]) if same < 48 else (old_once, new_once)
                    i = next((i for i in range(min(len(a), len(b))) if a[i] != b[i]), min(len(a), len(b)))
                    print("    first differs at byte %d:\n    old %r\n    new %r" % (i, a[i - 80:i + 80], b[i - 80:i + 80]))
finally:
    if tmux("ls", check=False).returncode == 0:
        tmux("kill-server")
    if os.environ.get("KEEP"):
        print("kept", tmp)
print("all identical" if failures == 0 else "%d runs differ" % failures)
raise SystemExit(failures != 0)
