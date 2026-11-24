//! The static board: walls, blocked centre cells and targets.

use std::fmt;

use crate::geometry::{BOARD_SIZE, Direction, Position};
use crate::pieces::Target;

const CELLS: usize = (BOARD_SIZE as usize) * (BOARD_SIZE as usize);

/// Walls around one cell, one bit per side.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct Walls(u8);

impl Walls {
    const fn bit(dir: Direction) -> u8 {
        match dir {
            Direction::Up => 1,
            Direction::Down => 1 << 1,
            Direction::Left => 1 << 2,
            Direction::Right => 1 << 3,
        }
    }

    const fn has(self, dir: Direction) -> bool {
        self.0 & Self::bit(dir) != 0
    }

    fn add(&mut self, dir: Direction) {
        self.0 |= Self::bit(dir);
    }
}

/// An immutable board. Built only through [`BoardBuilder`], which keeps
/// walls consistent: a wall is always seen from both cells it separates.
/// The board edge always stops movement and needs no explicit walls.
#[derive(Clone, PartialEq, Eq)]
pub struct Board {
    walls: [Walls; CELLS],
    blocked: [bool; CELLS],
    targets: Vec<(Position, Target)>,
}

impl Board {
    /// Whether a wall stops movement out of `pos` towards `dir`.
    #[must_use]
    pub fn has_wall(&self, pos: Position, dir: Direction) -> bool {
        self.walls[pos.index()].has(dir)
    }

    /// Whether robots may never enter `pos` (the centre block).
    #[must_use]
    pub fn is_blocked(&self, pos: Position) -> bool {
        self.blocked[pos.index()]
    }

    #[must_use]
    pub fn target_at(&self, pos: Position) -> Option<Target> {
        self.targets
            .iter()
            .find(|(p, _)| *p == pos)
            .map(|(_, t)| *t)
    }

    #[must_use]
    pub fn position_of(&self, target: Target) -> Option<Position> {
        self.targets
            .iter()
            .find(|(_, t)| *t == target)
            .map(|(p, _)| *p)
    }

    /// Targets and their cells, in placement order.
    #[must_use]
    pub fn targets(&self) -> &[(Position, Target)] {
        &self.targets
    }
}

impl fmt::Debug for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Board")
            .field("targets", &self.targets)
            .finish_non_exhaustive()
    }
}

/// Why a board could not be built.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoardError {
    CellHasTarget(Position),
    TargetPlacedTwice(Target),
    TargetOnBlockedCell(Position),
}

impl fmt::Display for BoardError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CellHasTarget(p) => write!(f, "cell {p:?} already has a target"),
            Self::TargetPlacedTwice(t) => write!(f, "target {t:?} is placed twice"),
            Self::TargetOnBlockedCell(p) => write!(f, "target on blocked cell {p:?}"),
        }
    }
}

impl std::error::Error for BoardError {}

/// Builds a [`Board`]; see its docs for the guarantees.
#[derive(Debug, Clone)]
pub struct BoardBuilder {
    board: Board,
}

impl Default for BoardBuilder {
    fn default() -> Self {
        Self {
            board: Board {
                walls: [Walls::default(); CELLS],
                blocked: [false; CELLS],
                targets: Vec::new(),
            },
        }
    }
}

impl BoardBuilder {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a wall on the `dir` side of `pos`, and on the facing side of the
    /// neighbour. A wall along the board edge is accepted and changes nothing.
    #[must_use]
    pub fn wall(mut self, pos: Position, dir: Direction) -> Self {
        self.board.walls[pos.index()].add(dir);
        if let Some(neighbour) = pos.step(dir) {
            self.board.walls[neighbour.index()].add(dir.opposite());
        }
        self
    }

    /// Marks `pos` as a cell robots may never enter.
    #[must_use]
    pub fn block(mut self, pos: Position) -> Self {
        self.board.blocked[pos.index()] = true;
        self
    }

    /// Places `target` on `pos`.
    ///
    /// # Errors
    /// If the cell already has a target or the target is already placed.
    pub fn target(mut self, pos: Position, target: Target) -> Result<Self, BoardError> {
        if self.board.target_at(pos).is_some() {
            return Err(BoardError::CellHasTarget(pos));
        }
        if self.board.position_of(target).is_some() {
            return Err(BoardError::TargetPlacedTwice(target));
        }
        self.board.targets.push((pos, target));
        Ok(self)
    }

    /// # Errors
    /// If a target sits on a blocked cell.
    pub fn build(self) -> Result<Board, BoardError> {
        if let Some((pos, _)) = self
            .board
            .targets
            .iter()
            .find(|(p, _)| self.board.is_blocked(*p))
        {
            return Err(BoardError::TargetOnBlockedCell(*pos));
        }
        Ok(self.board)
    }
}
