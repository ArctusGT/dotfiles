#!/usr/bin/env python3
"""PROTOTYPE, throwaway: plays Clawd with a per-session bandana, running and
idle (stand, squat), in Unicode octants (2x4 pixels a cell) beside quarter
blocks (2x2, as claude-sidebar draws today), to judge both in Ghostty.

Frames are the Maintainer's mocks, 16 pixels wide and 12 tall; quarter
blocks draw them at half the height. Three sessions run side by side, each
with its own bandana colours. Ctrl-C stops. Changes nothing.
  --fps N    frames a second (default 8)
"""
import argparse, random, sys, time, unicodedata

# . empty, B body, m bandana main, 1 2 3 the bandana stripes (the ribbon
# takes one of them). Octants draw 2x4 pixel cells starting at rows 0, 4
# and 8, and a cell shows two colours at most, the background counting as
# one: frames keep every cell to that, so the bandana never shares a cell
# with both body and an eye or the background
STAND = """
..............3.
..............3.
..112233mmmmmm.3
..112233mmmmmm.3
..BB.BBBBBB.BB..
..BB.BBBBBB.BB..
BBBBBBBBBBBBBBBB
BBBBBBBBBBBBBBBB
..BBBBBBBBBBBB..
..BBBBBBBBBBBB..
..B.B......B.B..
..B.B......B.B..
"""
SQUAT_DOWN = """
................
...............2
...............2
..112233mmmmmm2.
..112233mmmmmm..
BBBBBBBBBBBBBBBB
BBBBBBBBBBBBBBBB
..BBBBBBBBBBBB..
..BB..BBBB..BB..
..BBBBBBBBBBBB..
..BBBBBBBBBBBB..
..B.B......B.B..
"""
# running: body shifted right, arms swept left, stripes on the right end,
# the ribbon streaming left in the cell of columns 0-1, rows 0-3 (R), in a
# stripe colour that changes each frame; the feet stamping
RUN_BODY = """
RR..............
RR..............
RR.mmmmmmm332211
RR.mmmmmmm332211
...BBBBBBBBBBBBB
.BBBB..BBBBBB..B
.BBBBBBBBBBBBBBB
...BBBBBBBBBBBBB
.BBBBBBBBBBBBBBB
.BBBBBBBBBBBBBBB
"""
RIBBONS = [("..", "x.", ".x", ".x"), (".x", "x.", "x.", ".."),
           ("..", ".x", "xx", ".."), ("x.", ".x", ".x", "x.")]
RIBBON_COLOUR = ["1", "2", "3", "2"]
# four steps: the left foot of each pair down, both up, the right down, both up
FEET = [["....B.B.....B.B.", "....B.......B..."],
        ["....B.B.....B.B.", "................"],
        ["....B.B.....B.B.", "......B.......B."],
        ["....B.B.....B.B.", "................"]]


def grid(text):
    return [list(r) for r in text.strip("\n").split("\n")]


def run_frame(k):
    rows = grid(RUN_BODY) + [list(FEET[k][0]), list(FEET[k][1])]
    for y in range(4):
        for x in range(2):
            rows[y][x] = RIBBON_COLOUR[k] if RIBBONS[k][y][x] == "x" else "."
    return rows


# TokyoNight truecolour; body by state, bandanas from the rest
ORANGE, GREY = (255, 158, 100), (115, 122, 162)
BAND = [(247, 118, 142), (122, 162, 247), (158, 206, 106), (125, 207, 255),
        (187, 154, 247), (115, 218, 202), (224, 175, 104), (192, 202, 245),
        (42, 195, 222), (157, 124, 216)]

OCTANT = {0: " ", 255: "█"}
for cp in range(0x1CD00, 0x1CDE6):
    digits = unicodedata.name(chr(cp)).split("-")[1]
    OCTANT[sum(1 << (int(d) - 1) for d in digits)] = chr(cp)
# patterns Unicode draws with older characters: quadrants and quarter blocks
QUAD = " ▘▝▀▖▌▞▛▗▚▐▜▄▙▟█"
for q in range(16):
    bits = 0
    if q & 1: bits |= 0b00000101  # upper left: octants 1, 3
    if q & 2: bits |= 0b00001010  # upper right: 2, 4
    if q & 4: bits |= 0b01010000  # lower left: 5, 7
    if q & 8: bits |= 0b10100000  # lower right: 6, 8
    OCTANT.setdefault(bits, QUAD[q])
for bits, ch in {0b11: "\U0001FB82", 0b11000000: "▂",
                 0b111111: "\U0001FB85", 0b11111100: "▆",
                 0b1: "\U0001CEA8", 0b10: "\U0001CEAB", 0b1000000: "\U0001CEA3",
                 0b10000000: "\U0001CEA0", 0b10100: "\U0001FBE6",
                 0b101000: "\U0001FBE7"}.items():
    OCTANT.setdefault(bits, ch)


def sgr(fg, bg):
    s = "\x1b[38;2;%d;%d;%dm" % fg
    return s + ("\x1b[48;2;%d;%d;%dm" % bg if bg else "")


def render(px, cell_h, glyph):
    """Lines of cells 2 wide and cell_h tall; each cell keeps its two most
    common colours, the rest counted as clashes."""
    lines, clashes = [], 0
    for top in range(0, len(px), cell_h):
        out = ""
        for x in range(0, 16, 2):
            cells = [px[top + dy][x + dx] for dy in range(cell_h) for dx in (0, 1)]
            lit = [c for c in cells if c]
            if not lit:
                out += " "
                continue
            ranked = sorted(set(lit), key=lit.count, reverse=True)
            clashes += len(ranked) > 2 or (len(ranked) == 2 and len(lit) < len(cells))
            fg = ranked[0]
            bg = ranked[1] if len(ranked) > 1 and len(lit) == len(cells) else None
            bits = sum(1 << i for i, c in enumerate(cells) if c == fg)
            out += sgr(fg, bg) + glyph(bits) + "\x1b[0m"
        lines.append(out)
    return lines, clashes


def octants(px):
    return render(px, 4, lambda b: OCTANT.get(b, "?"))


def quarters(px):
    # half the height: each quarter-block pixel takes the upper of two rows
    half = [[a or b for a, b in zip(px[y], px[y + 1])] for y in range(0, 12, 2)]
    return render(half, 2, lambda b: QUAD[b])


def paint(frame, body, band):
    colours = {"B": body, "m": band[0], "1": band[1], "2": band[2], "3": band[3]}
    return [[colours.get(c) for c in row] for row in frame]


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--fps", type=float, default=8)
    args = ap.parse_args()
    sessions = [random.Random(s).sample(BAND, 4) for s in ("dotfiles-a", "ssci-b", "slurm-c")]
    idle = [grid(STAND)] * 12 + [grid(SQUAT_DOWN)] * 3
    sys.stdout.write("\x1b[2J\x1b[?25l")
    try:
        t = 0
        while True:
            run = run_frame(t % 4)
            still = idle[t % len(idle)]
            out = ["\x1b[H  running: octants, then quarter blocks, three sessions",
                   "  (\x1b[1m%d fps\x1b[0m; ctrl-c stops)" % args.fps, ""]
            for label, draw in (("octants", octants), ("quarters", quarters)):
                blocks, clashes = [], 0
                for band in sessions:
                    for frame, body in ((run, ORANGE), (still, GREY)):
                        lines, c = draw(paint(frame, body, band))
                        clashes += c
                        blocks.append(lines)
                out.append("  %-9s cells with more than two colours: %d   " % (label, clashes))
                for y in range(3):
                    out.append("  " + "   ".join(b[y] for b in blocks))
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
