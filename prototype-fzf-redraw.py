#!/usr/bin/env python3
"""PROTOTYPE, throwaway: animates sidebar-shaped entries in fzf by posting
reload-sync to its --listen socket, as claude-sidebar's serve loop does, to
see whether fzf flickers or loses the cursor at the animation's frame rate.

Draws the Clawd frames from prototype-clawd-run.py (beside this file) in
octants: a heading, then entries of three picture lines and a bar, about
half running and half idle. Move the cursor and click while it runs; Esc
or ctrl-c quits, then a summary prints: reload latency, missed frames, and
fzf's CPU. Changes nothing outside ${TMPDIR:-/tmp}/fzf-redraw-prototype.
  --fps N      frames a second (default 8)
  --entries N  entries in the list (default 11)
"""
import argparse, http.client, importlib.util, os, random, socket, subprocess
import sys, threading, time

here = os.path.dirname(os.path.abspath(__file__))
spec = importlib.util.spec_from_file_location("clawd", os.path.join(here, "prototype-clawd-run.py"))
clawd = importlib.util.module_from_spec(spec)
spec.loader.exec_module(clawd)


class UnixHTTP(http.client.HTTPConnection):
    def __init__(self, path):
        super().__init__("localhost")
        self.path = path

    def connect(self):
        self.sock = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
        self.sock.connect(self.path)


def entry(n, running, band, t, phase):
    if running:
        frame, body = clawd.run_frame(t % 4), clawd.ORANGE
    else:
        idle = [clawd.grid(clawd.STAND)] * 12 + [clawd.grid(clawd.SQUAT_DOWN)] * 3
        frame, body = idle[(t + phase) % len(idle)], clawd.GREY
    pic, _ = clawd.octants(clawd.paint(frame, body, band))
    text = ["%d repo-%d" % (n, n), "session name", "working" if running else "idle 3m"]
    bar = "\x1b[32m" + "━" * (6 + 3 * n) + "\x1b[90m" + "┈" * (34 - 3 * n) + "\x1b[0m"
    return "\n".join(" " + pic[i] + "  " + text[i] for i in range(3)) + "\n " + bar


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--fps", type=float, default=8)
    ap.add_argument("--entries", type=int, default=11)
    args = ap.parse_args()
    work = os.path.join(os.environ.get("TMPDIR", "/tmp"), "fzf-redraw-prototype")
    os.makedirs(work, exist_ok=True)
    sock, rows = os.path.join(work, "fzf.sock"), os.path.join(work, "rows")
    rng = random.Random(7)
    sessions = [(i + 1, i % 2 == 0, rng.sample(clawd.BAND, 4), rng.randrange(15))
                for i in range(args.entries)]

    def write(t):
        items = ["\x1b[1;34mlocal\x1b[0m\n⠀"] + [entry(n, r, b, t, ph) for n, r, b, ph in sessions]
        with open(rows + ".new", "w") as f:
            f.write("\0".join(items) + "\0")
        os.replace(rows + ".new", rows)

    write(0)
    if os.path.exists(sock):
        os.remove(sock)
    fzf = subprocess.Popen(["fzf", "--read0", "--ansi", "--listen", sock, "--layout", "reverse",
                            "--no-scrollbar", "--info", "hidden", "--prompt", "",
                            "--bind", "esc,ctrl-c:abort"], stdin=open(rows))
    stats = {"latency": [], "late": 0, "cpu": []}
    stop = threading.Event()

    def animate():
        while not os.path.exists(sock) and not stop.is_set():
            time.sleep(0.05)
        t, period, next_at = 0, 1 / args.fps, time.monotonic()
        while not stop.is_set():
            t += 1
            write(t)
            start = time.monotonic()
            try:
                c = UnixHTTP(sock)
                c.request("POST", "/", body="reload-sync(cat '%s')" % rows)
                c.getresponse().read()
                c.close()
            except OSError:
                break
            stats["latency"].append(time.monotonic() - start)
            next_at += period
            delay = next_at - time.monotonic()
            if delay < 0:
                stats["late"] += 1
                next_at = time.monotonic()
            else:
                stop.wait(delay)

    def sample():
        while not stop.wait(1):
            out = subprocess.run(["ps", "-o", "%cpu=", "-p", str(fzf.pid)], capture_output=True, text=True)
            if out.stdout.strip():
                stats["cpu"].append(float(out.stdout))

    threads = [threading.Thread(target=f, daemon=True) for f in (animate, sample)]
    for th in threads:
        th.start()
    fzf.wait()
    stop.set()
    lat = sorted(stats["latency"]) or [0]
    print("frames posted: %d at %g fps; late: %d" % (len(stats["latency"]), args.fps, stats["late"]))
    print("reload latency ms: median %.1f, p95 %.1f, max %.1f" % (
        1000 * lat[len(lat) // 2], 1000 * lat[int(len(lat) * 0.95)], 1000 * lat[-1]))
    if stats["cpu"]:
        print("fzf cpu %%: mean %.0f, max %.0f" % (sum(stats["cpu"]) / len(stats["cpu"]), max(stats["cpu"])))


if __name__ == "__main__":
    main()
