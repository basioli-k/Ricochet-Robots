//! Game rules for Ricochet Robots: board, movement, bidding and scoring.
//!
//! Pure by design: no I/O, async or framework types, so every rule can be
//! unit-tested and reused by the server, the solver and client previews.

mod bidding;
mod board;
pub mod fixed_board;
mod game;
mod geometry;
mod movement;
mod pieces;
mod robots;
mod round;
mod solver;
mod validator;

pub use bidding::{Bid, BidError, Bids, PlayerId, Timestamp};
pub use board::{Board, BoardBuilder, BoardError};
pub use game::{Game, GameError, Shuffle};
pub use geometry::{BOARD_SIZE, Direction, Position};
pub use pieces::{RobotColor, Symbol, Target};
pub use robots::{Robots, SharedCell};
pub use round::{Phase, Round, RoundError};
pub use solver::{Move, solve};
pub use validator::{BoardProblem, BoardValidator};
