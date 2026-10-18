#!/usr/bin/env python3
"""PROTOTYPE, throwaway: the bandana one row lower on Clawd, standing and
running, beside today's standing Clawd and the Maintainer's mock drawn as is,
in octants, for three sessions. Each picture is labelled with its count of
cells needing more than two colours (pixels a cell cannot show).

Draws with prototype-clawd-run.py beside this file. Ctrl-C stops. Changes
nothing.
  --fps N    frames a second (default 8)
"""
import argparse, importlib.util, os, random, sys, time

here = os.path.dirname(os.path.abspath(__file__))
spec = importlib.util.spec_from_file_location("clawd", os.path.join(here, "prototype-clawd-run.py"))
c = importlib.util.module_from_spec(spec)
spec.loader.exec_module(c)

# the Maintainer's mock, as drawn: head-top row, bandana, eyes, a body row,
# arms, body, feet
MOCK = """
................
................
..BBBBBBBBBBBB..
..112233mmmmmm3.
..112233mmmmmm3.
..BB.BBBBBB.BB.2
..BB.BBBBBB.BB.2
..BBBBBBBBBBBB..
BBBBBBBBBBBBBBBB
BBBBBBBBBBBBBBBB
..BBBBBBBBBBBB..
..B.B......B.B..
"""
# the same idea on whole cells: every feature two rows, head-top, bandana,
# eyes, arms, body, feet; the ribbon a tail at the bandana's end and a
# two-wide drop beside the eyes, one colour a cell
LOW_STAND = """
..BBBBBBBBBBBB..
..BBBBBBBBBBBB..
..112233mmmmmm3.
..112233mmmmmm3.
..BB.BBBBBB.BB22
..BB.BBBBBB.BB22
BBBBBBBBBBBBBBBB
BBBBBBBBBBBBBBBB
..BBBBBBBBBBBB..
..BBBBBBBBBBBB..
..B.B......B.B..
..B.B......B.B..
"""
# running: the body two columns right (4-15), stripes at the far end, slit
# eyes, both arms swept left; the ribbon streams in the two cells on the
# left (columns 0-1 and 2-3, rows 0-3), one stripe colour each
LOW_RUN = """
....BBBBBBBBBBBB
....BBBBBBBBBBBB
....mmmmmm332211
....mmmmmm332211
....BB..BBBB..BB
.BB.BBBBBBBBBBBB
.BB.BBBBBBBBBBBB
....BBBBBBBBBBBB
.BBBBBBBBBBBBBBB
.BBBBBBBBBBBBBBB
"""
RIBBON_L = [("..", "x.", ".x", ".x"), (".x", "x.", "x.", ".."),
            ("..", ".x", "xx", ".."), ("x.", ".x", ".x", "x.")]
RIBBON_R = [("..", "..", "xx", "x."), ("..", "x.", "xx", ".."),
            ("..", "..", "xx", ".x"), (".x", "x.", "xx", "..")]
RIBBON_COLOURS = [("1", "3"), ("2", "1"), ("3", "2"), ("2", "3")]
# four steps: the left foot of each pair down, both up, the right down, both up
LOW_FEET = [[".....B.B....B.B.", ".....B......B..."],
            [".....B.B....B.B.", "................"],
            [".....B.B....B.B.", ".......B......B."],
            [".....B.B....B.B.", "................"]]


def low_run(k):
    rows = c.grid(LOW_RUN) + [list(LOW_FEET[k][0]), list(LOW_FEET[k][1])]
    for y in range(4):
        for x in range(2):
            rows[y][x] = RIBBON_COLOURS[k][0] if RIBBON_L[k][y][x] == "x" else "."
            rows[y][x + 2] = RIBBON_COLOURS[k][1] if RIBBON_R[k][y][x] == "x" else rows[y][x + 2]
    return rows


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--fps", type=float, default=8)
    args = ap.parse_args()
    sessions = [random.Random(s).sample(c.BAND, 4) for s in ("dotfiles-a", "ssci-b", "slurm-c")]
    pictures = [("today", lambda t: c.grid(c.STAND), c.GREY), ("mock as is", lambda t: c.grid(MOCK), c.GREY),
                ("low stand", lambda t: c.grid(LOW_STAND), c.GREY), ("low run", lambda t: low_run(t % 4), c.ORANGE)]
    sys.stdout.write("\x1b[2J\x1b[?25l")
    try:
        t = 0
        while True:
            out = ["\x1b[H  bandana one row lower (%g fps; ctrl-c stops)" % args.fps, "",
                   "  " + "".join("%-14s" % name for name, _, _ in pictures)]
            for band in sessions:
                blocks, labels = [], []
                for name, frame, body in pictures:
                    lines, clashes = c.octants(c.paint(frame(t), body, band))
                    blocks.append(lines)
                    labels.append("%-14s" % ("clashes %d" % clashes))
                for y in range(3):
                    out.append("  " + "      ".join(b[y] for b in blocks))
                out.append("  " + "".join(labels))
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
