//! The single fixed board, ported from the legacy PHP app
//! (`legacy/controller/gameplayController.php`).
//!
//! Walls keep the legacy per-cell encoding so the data can be diffed against
//! the original: four digits per cell for the left, top, right and bottom
//! sides. The legacy data has no one-sided walls and a closed outer edge.

use crate::board::{Board, BoardBuilder};
use crate::geometry::{Direction, Position};
use crate::pieces::RobotColor::{Blue, Green, Red, Yellow};
use crate::pieces::Symbol::{Gear, Moon, Planet, Star};
use crate::pieces::Target;
use crate::robots::Robots;

#[rustfmt::skip]
const WALLS: [&str; 16] = [
    "1100 0110 1100 0100 0101 0100 0100 0100 0100 0110 1100 0100 0100 0100 0101 0110",
    "1000 0001 0000 0010 1100 0000 0000 0000 0000 0000 0000 0000 0000 0010 1100 0010",
    "1000 0110 1000 0000 0000 0000 0000 0000 0000 0000 0010 1001 0000 0000 0000 0010",
    "1000 0000 0000 0000 0000 0000 0011 1000 0000 0000 0000 0100 0000 0000 0000 0011",
    "1000 0000 0000 0000 0000 0000 0100 0000 0000 0000 0000 0000 0000 0000 0000 0110",
    "1001 0000 0000 0000 0000 0000 0000 0000 0000 0000 0000 0000 0000 0000 0000 0010",
    "1100 0000 0010 1001 0000 0000 0000 0001 0001 0000 0001 0000 0000 0011 1000 0010",
    "1000 0000 0000 0100 0000 0001 0010 1100 0110 1000 0110 1000 0000 0100 0000 0010",
    "1000 0000 0000 0000 0000 0110 1010 1001 0011 1000 0000 0000 0000 0000 0000 0010",
    "1000 0011 1000 0000 0000 0000 0000 0100 0100 0000 0000 0000 0010 1001 0000 0010",
    "1000 0100 0000 0010 1001 0000 0000 0000 0000 0000 0000 0000 0000 0100 0000 0010",
    "1001 0000 0000 0000 0100 0000 0000 0000 0000 0011 1000 0000 0000 0000 0000 0011",
    "1100 0000 0000 0000 0000 0001 0000 0000 0000 0100 0000 0000 0000 0000 0001 0110",
    "1000 0000 0000 0001 0000 0110 1000 0000 0000 0000 0001 0000 0000 0000 0110 1010",
    "1000 0000 0010 1100 0000 0000 0000 0000 0000 0010 1100 0000 0000 0000 0000 0010",
    "1001 0001 0001 0001 0001 0001 0011 1001 0001 0001 0001 0011 1001 0001 0001 0011",
];

/// Side order of the four digits in [`WALLS`].
const SIDES: [Direction; 4] = [
    Direction::Left,
    Direction::Up,
    Direction::Right,
    Direction::Down,
];

#[rustfmt::skip]
const TARGETS: [(u8, u8, Target); 17] = [
    (1, 4, Target::Colored(Red, Moon)),
    (1, 14, Target::Colored(Red, Gear)),
    (2, 1, Target::Colored(Green, Gear)),
    (2, 11, Target::Colored(Blue, Star)),
    (3, 6, Target::Colored(Yellow, Star)),
    (6, 3, Target::Colored(Blue, Planet)),
    (6, 13, Target::Colored(Green, Moon)),
    (7, 10, Target::Colored(Yellow, Planet)),
    (8, 5, Target::Vortex),
    (9, 1, Target::Colored(Blue, Moon)),
    (9, 13, Target::Colored(Blue, Gear)),
    (10, 4, Target::Colored(Green, Planet)),
    (11, 9, Target::Colored(Yellow, Moon)),
    (13, 5, Target::Colored(Red, Star)),
    (13, 14, Target::Colored(Red, Planet)),
    (14, 3, Target::Colored(Yellow, Gear)),
    (14, 10, Target::Colored(Green, Star)),
];

/// The 2x2 centre block robots can't enter.
const BLOCKED: [(u8, u8); 4] = [(7, 7), (7, 8), (8, 7), (8, 8)];

/// Start cells, indexed like `RobotColor::ALL` (red, blue, green, yellow).
const START: [(u8, u8); 4] = [(1, 1), (15, 13), (14, 0), (4, 0)];

fn cell(row: u8, col: u8) -> Position {
    Position::new(row, col).expect("fixed board data is in bounds")
}

/// The fixed board.
///
/// # Panics
/// Never for the bundled data; its tests build it.
#[must_use]
pub fn board() -> Board {
    let mut builder = BoardBuilder::new();
    for (row, codes) in (0..).zip(WALLS) {
        for (col, code) in (0..).zip(codes.split(' ')) {
            for (side, digit) in SIDES.into_iter().zip(code.bytes()) {
                if digit == b'1' {
                    builder = builder.wall(cell(row, col), side);
                }
            }
        }
    }
    for (row, col) in BLOCKED {
        builder = builder.block(cell(row, col));
    }
    for (row, col, target) in TARGETS {
        builder = builder
            .target(cell(row, col), target)
            .expect("fixed board targets are distinct");
    }
    builder
        .build()
        .expect("fixed board targets avoid the centre")
}

/// Where the robots start on the fixed board.
///
/// # Panics
/// Never for the bundled data; its tests check it.
#[must_use]
pub fn starting_robots() -> Robots {
    Robots::new(START.map(|(row, col)| cell(row, col))).expect("start cells are distinct")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn digit(row: u8, col: u8, side: Direction) -> bool {
        let code = WALLS[usize::from(row)]
            .split(' ')
            .nth(usize::from(col))
            .unwrap();
        let i = SIDES.iter().position(|s| *s == side).unwrap();
        code.as_bytes()[i] == b'1'
    }

    /// The builder would silently complete a one-sided wall, so guard the
    /// raw data: every wall must be written on both cells it separates.
    #[test]
    fn wall_data_is_written_on_both_sides() {
        for pos in Position::all() {
            for side in SIDES {
                if let Some(n) = pos.step(side) {
                    assert_eq!(
                        digit(pos.row(), pos.col(), side),
                        digit(n.row(), n.col(), side.opposite()),
                        "wall between {pos:?} and {n:?} is one-sided",
                    );
                }
            }
        }
    }

    #[test]
    fn every_row_has_sixteen_four_digit_codes() {
        for codes in WALLS {
            let cells: Vec<_> = codes.split(' ').collect();
            assert_eq!(cells.len(), 16);
            assert!(
                cells
                    .iter()
                    .all(|c| c.len() == 4 && c.bytes().all(|b| b == b'0' || b == b'1'))
            );
        }
    }
}
