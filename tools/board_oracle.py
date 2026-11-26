"""Expected values for crates/domain/tests/fixed_board.rs, from the legacy app.

Parses the legacy board literal kept in tools/legacy_board.php and replays
moves with the legacy game's rule (from its robot_movement.js): a robot
moves until the side it is moving towards has a wall, stepping back if it
would enter a cell holding another robot. Independent of the Rust code on
purpose.

Usage, from the repository root:  python3 -I tools/board_oracle.py
"""

import random
import re
from pathlib import Path

PHP = Path(__file__).resolve().parent / "legacy_board.php"
COLORS = {"RED": "Red", "BLUE": "Blue", "GREEN": "Green", "YELLOW": "Yellow"}
SYMBOLS = {"STAR": "Star", "PLANET": "Planet", "GEAR": "Gear", "MOON": "Moon"}
# Direction -> (row step, col step, index of that side in the legacy code "LTRB").
DIRS = {"Up": (-1, 0, 1), "Down": (1, 0, 3), "Left": (0, -1, 0), "Right": (0, 1, 2)}
FIELD = re.compile(
    r'new Field\("([01]{4})"(?:\s*,\s*(NULL|array\((\w+),\s*(\w+)\))\s*(?:,\s*(NULL|\w+))?)?\)'
)


def parse():
    src = PHP.read_text()
    start = src.index("$board = [")
    body = src[start : src.index("];", start)]
    rows = re.findall(r"\[(new Field.*?)\](?=,\s*\n|\s*$)", body, re.S)
    grid = [FIELD.findall(row) for row in rows]
    assert len(grid) == 16 and all(len(row) == 16 for row in grid)
    return grid


def slide(grid, robots, name, direction):
    di, dj, side = DIRS[direction]
    row, col = robots[name]
    others = {pos for other, pos in robots.items() if other != name}
    while grid[row][col][0][side] != "1":
        if (row + di, col + dj) in others:
            break
        row, col = row + di, col + dj
        # Python would wrap a negative index to the far side of the board.
        assert 0 <= row < 16 and 0 <= col < 16, "outer edge is not walled"
    return row, col


def main():
    grid = parse()
    targets = [
        (i, j, "Vortex" if cell[3] == "VORTEX" else f"{COLORS[cell[2]]} {SYMBOLS[cell[3]]}")
        for i, row in enumerate(grid)
        for j, cell in enumerate(row)
        if cell[2]
    ]
    print("TARGETS")
    for i, j, target in targets:
        print(f"  ({i}, {j}) {target}")

    # Stop table: from every cell, alone on the board, where a robot stops.
    # One hex digit per cell: the stop row for Up/Down, the stop column for
    # Left/Right. Any wall added or removed changes at least one digit.
    print("STOPS (16 rows of 16 cells per direction)")
    for direction in ("Up", "Down", "Left", "Right"):
        index = 0 if direction in ("Up", "Down") else 1
        print(f"  {direction}")
        for i in range(16):
            stops = (slide(grid, {"r": (i, j)}, "r", direction)[index] for j in range(16))
            print("    " + "".join(f"{s:x}" for s in stops))

    start = {COLORS[c[4]]: (i, j) for i, row in enumerate(grid) for j, c in enumerate(row) if c[4] in COLORS}
    print("FIRST MOVES (from the start position)")
    for name in ("Red", "Blue", "Green", "Yellow"):
        for direction in ("Up", "Down", "Left", "Right"):
            print(f"  {name}:{direction} -> {slide(grid, start, name, direction)}")
    rng = random.Random(2026)  # fixed seed: the output must be reproducible
    print("SEQUENCES (moves, then final red/blue/green/yellow cells)")
    for _ in range(8):
        robots = dict(start)
        moves = [(rng.choice(sorted(COLORS.values())), rng.choice(sorted(DIRS))) for _ in range(12)]
        for name, direction in moves:
            robots[name] = slide(grid, robots, name, direction)
        final = [robots[name] for name in ("Red", "Blue", "Green", "Yellow")]
        print("  " + " ".join(f"{n}:{d}" for n, d in moves) + " -> " + " ".join(map(str, final)))


if __name__ == "__main__":
    main()
