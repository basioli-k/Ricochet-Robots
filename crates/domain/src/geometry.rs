//! Cells and directions on the square board.

/// Number of rows and columns on the board.
pub const BOARD_SIZE: u8 = 16;

/// A cell on the board; always within bounds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Position {
    row: u8,
    col: u8,
}

impl Position {
    /// Returns `None` when the cell is outside the board.
    #[must_use]
    pub const fn new(row: u8, col: u8) -> Option<Self> {
        if row < BOARD_SIZE && col < BOARD_SIZE {
            Some(Self { row, col })
        } else {
            None
        }
    }

    #[must_use]
    pub const fn row(self) -> u8 {
        self.row
    }

    #[must_use]
    pub const fn col(self) -> u8 {
        self.col
    }

    /// The neighbouring cell in `dir`, or `None` at the board edge.
    #[must_use]
    pub fn step(self, dir: Direction) -> Option<Self> {
        let (row, col) = match dir {
            Direction::Up => (self.row.checked_sub(1)?, self.col),
            Direction::Down => (self.row + 1, self.col),
            Direction::Left => (self.row, self.col.checked_sub(1)?),
            Direction::Right => (self.row, self.col + 1),
        };
        Self::new(row, col)
    }

    /// Every cell, row by row.
    pub fn all() -> impl Iterator<Item = Self> {
        (0..BOARD_SIZE).flat_map(|row| (0..BOARD_SIZE).map(move |col| Self { row, col }))
    }
}

/// A direction a robot can move in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    pub const ALL: [Self; 4] = [Self::Up, Self::Down, Self::Left, Self::Right];

    #[must_use]
    pub const fn opposite(self) -> Self {
        match self {
            Self::Up => Self::Down,
            Self::Down => Self::Up,
            Self::Left => Self::Right,
            Self::Right => Self::Left,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pos(row: u8, col: u8) -> Position {
        Position::new(row, col).unwrap()
    }

    #[test]
    fn rejects_cells_outside_the_board() {
        assert!(Position::new(BOARD_SIZE, 0).is_none());
        assert!(Position::new(0, BOARD_SIZE).is_none());
        assert!(Position::new(BOARD_SIZE - 1, BOARD_SIZE - 1).is_some());
    }

    #[test]
    fn steps_to_neighbours() {
        let p = pos(5, 5);
        assert_eq!(p.step(Direction::Up), Some(pos(4, 5)));
        assert_eq!(p.step(Direction::Down), Some(pos(6, 5)));
        assert_eq!(p.step(Direction::Left), Some(pos(5, 4)));
        assert_eq!(p.step(Direction::Right), Some(pos(5, 6)));
    }

    #[test]
    fn stepping_off_any_edge_gives_none() {
        let last = BOARD_SIZE - 1;
        assert_eq!(pos(0, 3).step(Direction::Up), None);
        assert_eq!(pos(last, 3).step(Direction::Down), None);
        assert_eq!(pos(3, 0).step(Direction::Left), None);
        assert_eq!(pos(3, last).step(Direction::Right), None);
    }

    #[test]
    fn stepping_back_returns_to_the_start() {
        let p = pos(7, 9);
        for dir in Direction::ALL {
            assert_eq!(p.step(dir).and_then(|n| n.step(dir.opposite())), Some(p));
        }
    }

    #[test]
    fn all_lists_each_cell_once_row_by_row() {
        let cells: Vec<_> = Position::all().collect();
        assert_eq!(cells.len(), usize::from(BOARD_SIZE).pow(2));
        assert!(cells.windows(2).all(|w| w[0] < w[1]));
    }
}
