//! Game rules for Ricochet Robots: board, movement, bidding and scoring.
//!
//! Pure by design: no I/O, async or framework types, so every rule can be
//! unit-tested and reused by the server, the solver and client previews.

mod geometry;
mod pieces;

pub use geometry::{BOARD_SIZE, Direction, Position};
pub use pieces::{RobotColor, Symbol, Target};
