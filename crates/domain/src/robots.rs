//! Where the four robots stand.

use std::fmt;

use crate::geometry::Position;
use crate::pieces::RobotColor;

/// Positions of all four robots, always on distinct cells.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Robots([Position; 4]);

/// Two robots were placed on the same cell.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SharedCell(pub Position);

impl fmt::Display for SharedCell {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "two robots on cell {:?}", self.0)
    }
}

impl std::error::Error for SharedCell {}

impl Robots {
    /// `positions` is indexed by [`RobotColor::index`].
    ///
    /// # Errors
    /// If two robots share a cell.
    pub fn new(positions: [Position; 4]) -> Result<Self, SharedCell> {
        for (i, a) in positions.iter().enumerate() {
            if positions[i + 1..].contains(a) {
                return Err(SharedCell(*a));
            }
        }
        Ok(Self(positions))
    }

    #[must_use]
    pub const fn position(&self, robot: RobotColor) -> Position {
        self.0[robot.index()]
    }

    #[must_use]
    pub fn robot_at(&self, pos: Position) -> Option<RobotColor> {
        RobotColor::ALL
            .into_iter()
            .find(|r| self.position(*r) == pos)
    }

    /// Moves one robot. Callers guarantee `to` is free, so cells stay distinct.
    pub(crate) fn moved(mut self, robot: RobotColor, to: Position) -> Self {
        debug_assert!(self.robot_at(to).is_none_or(|r| r == robot));
        self.0[robot.index()] = to;
        self
    }
}
