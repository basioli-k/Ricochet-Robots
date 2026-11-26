"""The oracle still reads the legacy board it regenerates test values from."""

import unittest

import board_oracle as oracle


class ParseLegacyBoard(unittest.TestCase):
    def test_reads_a_walled_16_by_16_board_with_17_targets(self):
        grid = oracle.parse()
        self.assertEqual([len(row) for row in grid], [16] * 16)
        targets = [cell for row in grid for cell in row if cell[2]]
        self.assertEqual(len(targets), 17)
        # Each top-row cell has its top wall ("LTRB" order: index 1).
        self.assertTrue(all(cell[0][1] == "1" for cell in grid[0]))

    def test_a_robot_alone_slides_to_a_wall(self):
        grid = oracle.parse()
        robots = {"Red": (0, 0)}
        self.assertEqual(oracle.slide(grid, robots, "Red", "Up"), (0, 0))


if __name__ == "__main__":
    unittest.main()
