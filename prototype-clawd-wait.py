#!/usr/bin/env python3
"""PROTOTYPE, throwaway: waiting animations for Clawd (state waiting, body
yellow), side by side in octants for three sessions, each session at its own
phase as the sidebar staggers the squat. Starts from main's standing Clawd
(lower bandana, slit eyes). Each column is labelled with its loop length
in frames and its count of cells needing more than two colours.

Draws with prototype-clawd-run.py beside this file. Ctrl-C stops. Changes
nothing.
  --fps N    frames a second (default 8, as the sidebar)
"""
import argparse, importlib.util, os, random, sys, time

here = os.path.dirname(os.path.abspath(__file__))
spec = importlib.util.spec_from_file_location("clawd", os.path.join(here, "prototype-clawd-run.py"))
c = importlib.util.module_from_spec(spec)
spec.loader.exec_module(c)

YELLOW = (224, 175, 104)

# main's stand (claude-sidebar, after multi-host-agents/027)
STAND = """
..BBBBBBBBBBBB..
..BBBBBBBBBBBB..
..112233mmmmmm3.
..112233mmmmmm3.
..BB..BBBB..BB22
..BBBBBBBBBBBB22
BBBBBBBBBBBBBBBB
BBBBBBBBBBBBBBBB
..BBBBBBBBBBBB..
..BBBBBBBBBBBB..
..B.B......B.B..
..B.B......B.B..
"""

# the eye row (row 4): every cell there is body and empty, so the slits
# can move a pixel either way, or close, without a clash
EYES = {"centre": "..BB..BBBB..BB22", "left": "..B..BBBB..BBB22",
        "right": "..BBB..BBBB..B22", "shut": "..BBBBBBBBBBBB22"}
# the feet (rows 10-11): a pair lifted keeps only its upper row
FEET = {"down": ["..B.B......B.B..", "..B.B......B.B.."],
        "left up": ["..B.B......B.B..", "...........B.B.."],
        "right up": ["..B.B......B.B..", "..B.B..........."]}


def pose(eyes="centre", feet="down", left=0, right=0):
    """left/right arm: 0 down (rows 6-7), 1 half (rows 5-6), 2 up (rows 4-5),
    3 high (rows 2-3, left only: on the right the ribbon tail shares that
    cell). The right arm passes the ribbon drop, which hangs below it."""
    rows = c.grid(STAND)
    rows[4] = list(EYES[eyes])
    rows[10], rows[11] = list(FEET[feet][0]), list(FEET[feet][1])
    for y in range(2, 8):
        rows[y][0] = rows[y][1] = "."
    for y in {0: (6, 7), 1: (5, 6), 2: (4, 5), 3: (2, 3)}[left]:
        rows[y][0] = rows[y][1] = "B"
    arm = {0: (6, 7), 1: (5, 6), 2: (4, 5)}[right]
    for y in range(4, 8):
        rows[y][14] = rows[y][15] = "B" if y in arm else "2"
    return rows


def hold(p, n):
    return [p] * n


S = pose()
# 48 frames (6 s at 8 fps) unless named otherwise; the sidebar's frame_count
# is 12 today, so anything longer means raising it (to 48 covers run 4,
# squat 12 and these)
ANIMATIONS = [
    ("blink", hold(S, 30) + [pose("shut")] + hold(S, 4) + [pose("shut")] + hold(S, 12)),
    ("blink 12", hold(S, 11) + [pose("shut")]),
    ("look", hold(S, 8) + hold(pose("left"), 10) + hold(S, 4) + hold(pose("right"), 10)
             + hold(S, 8) + [pose("shut")] + hold(S, 7)),
    ("tap", hold(S, 24) + [pose(feet="right up"), S, pose(feet="right up"), S,
                           pose(feet="right up"), S] + hold(S, 18)),
    ("stamp 12", [pose(feet="left up"), S, S, pose(feet="right up"), S, S] + hold(S, 6)),
    ("arms up", hold(S, 24) + [pose(left=1, right=1)] + hold(pose(left=2, right=2), 8)
                + [pose(left=1, right=1)] + hold(S, 14)),
    ("wave", hold(S, 24) + [pose(left=1), pose(left=2)]
             + [pose(left=3), pose(left=2)] * 3 + [pose(left=1)] + hold(S, 15)),
    ("fidget", hold(S, 6) + hold(pose("left"), 6) + hold(S, 2) + hold(pose("right"), 6)
               + hold(S, 6) + [pose(feet="right up"), S, pose(feet="right up")]
               + hold(S, 8) + [pose("shut")] + hold(S, 4) + [pose(left=1), pose(left=2), pose(left=1)]
               + hold(S, 3)),
]
# the Maintainer's picks: look, stamp 12, wave. Eyes, feet and left arm sit
# in separate cells, so the picks combine without a clash
PICKS = ("look", "stamp 12", "wave")
ANIMATIONS = [a for a in ANIMATIONS if a[0] in PICKS]
LOOK = ["centre"] * 8 + ["left"] * 10 + ["centre"] * 4 + ["right"] * 10 + ["centre"] * 8 + ["shut"] + ["centre"] * 7
STAMP = ["left up", "down", "down", "right up", "down", "down"] + ["down"] * 6
WAVE = [0] * 24 + [1, 2] + [3, 2] * 3 + [1] + [0] * 15
# one after another in 48 frames: look about, stamp, wave
SEQ_EYES = ["centre"] * 4 + ["left"] * 6 + ["centre"] * 2 + ["right"] * 6 + ["centre"] * 30
SEQ_FEET = ["down"] * 20 + ["left up", "down", "down", "right up", "down", "down"] + ["down"] * 22
SEQ_ARM = [0] * 30 + [1, 2] + [3, 2] * 3 + [1] + [0] * 9
# all at once: each pick on its own loop, 48 frames being a multiple of each
ANIMATIONS += [
    ("in turn", [pose(SEQ_EYES[k], SEQ_FEET[k], SEQ_ARM[k]) for k in range(48)]),
    ("together", [pose(LOOK[k], STAMP[k % 12], WAVE[k]) for k in range(48)]),
]


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--fps", type=float, default=8)
    args = ap.parse_args()
    names = ("dotfiles-a", "ssci-b", "slurm-c")
    sessions = [(random.Random(s).sample(c.BAND, 4), random.Random(s).randrange(48)) for s in names]
    clashes = {name: sum(c.octants(c.paint(p, YELLOW, sessions[0][0]))[1] for p in poses)
               for name, poses in ANIMATIONS}
    # each column 14 wide: the 8-cell picture and a 6-wide gap, so the
    # columns wrap into as many rows as the pane's width fits
    width = 14
    sys.stdout.write("\x1b[?25l")
    size = None
    try:
        t = 0
        while True:
            if os.get_terminal_size() != size:
                size = os.get_terminal_size()
                sys.stdout.write("\x1b[2J")
            per_row = max(1, (size.columns - 2) // width)
            out = ["\x1b[H  waiting, %g fps, ctrl-c stops   frame %-4d" % (args.fps, t), ""]
            for i in range(0, len(ANIMATIONS), per_row):
                group = ANIMATIONS[i:i + per_row]
                out += ["  " + "".join("%-*s" % (width, name) for name, _ in group),
                        "  " + "".join("%-*s" % (width, "%d fr, %d cl" % (len(p), clashes[n])) for n, p in group),
                        ""]
                for band, phase in sessions:
                    blocks = [c.octants(c.paint(poses[(t + phase) % len(poses)], YELLOW, band))[0]
                              for _, poses in group]
                    for y in range(3):
                        out.append("  " + "      ".join(b[y] for b in blocks))
                    out.append("")
            sys.stdout.write("\n".join(out))
            sys.stdout.flush()
            t += 1
            time.sleep(1 / args.fps)
    except KeyboardInterrupt:
        pass
    finally:
        sys.stdout.write("\x1b[?25h\n")


if __name__ == "__main__":
    main()
