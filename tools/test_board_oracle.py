"""The oracle still reads the legacy board and agrees with the Rust tests."""

import unittest

import board_oracle as oracle


def start(grid):
    """Robot positions from the board literal, as the oracle's main() reads them."""
    return {
        oracle.COLORS[cell[4]]: (i, j)
        for i, row in enumerate(grid)
        for j, cell in enumerate(row)
        if cell[4] in oracle.COLORS
    }


class LegacyBoard(unittest.TestCase):
    def test_reads_a_16_by_16_board_with_17_targets_and_4_robots(self):
        grid = oracle.parse()
        self.assertEqual([len(row) for row in grid], [16] * 16)
        targets = [cell for row in grid for cell in row if cell[2]]
        self.assertEqual(len(targets), 17)
        self.assertEqual(sorted(start(grid)), ["Blue", "Green", "Red", "Yellow"])

    def test_slides_match_crates_domain_tests_fixed_board(self):
        # Values pinned in first_moves_match_the_legacy_game: if the copied
        # literal drifted, these would disagree.
        grid = oracle.parse()
        robots = start(grid)
        expected = {
            ("Yellow", "Right"): (4, 15),
            ("Green", "Up"): (12, 0),
            ("Blue", "Up"): (10, 13),
            ("Red", "Right"): (1, 3),
        }
        for (name, direction), stop in expected.items():
            self.assertEqual(oracle.slide(grid, robots, name, direction), stop, name)

    def test_a_robot_stops_before_another(self):
        grid = oracle.parse()
        robots = start(grid)
        red = robots["Red"]
        # Put Blue two cells right of Red: Red stops right next to it.
        robots["Blue"] = (red[0], red[1] + 2)
        self.assertEqual(oracle.slide(grid, robots, "Red", "Right"), (red[0], red[1] + 1))


if __name__ == "__main__":
    unittest.main()
