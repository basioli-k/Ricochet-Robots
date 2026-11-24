//! Game rules for Ricochet Robots: board, movement, bidding and scoring.
//!
//! Pure by design: no I/O, async or framework types, so every rule can be
//! unit-tested and reused by the server, the solver and client previews.

mod board;
pub mod fixed_board;
mod geometry;
mod movement;
mod pieces;
mod robots;

pub use board::{Board, BoardBuilder, BoardError};
pub use geometry::{BOARD_SIZE, Direction, Position};
pub use pieces::{RobotColor, Symbol, Target};
pub use robots::{Robots, SharedCell};
