//! One round: bid on a target, then the best bidders demonstrate in turn.

use std::fmt;

use crate::bidding::{Bid, BidError, Bids, PlayerId, Timestamp};
use crate::board::Board;
use crate::geometry::Direction;
use crate::pieces::{RobotColor, Target};
use crate::robots::Robots;

/// Where a round stands.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    /// Players bid; the first bid starts the window.
    Bidding,
    /// `player` must reach the target in at most `bid` moves.
    Demonstrating {
        player: PlayerId,
        bid: u8,
        moves_made: u8,
    },
    /// `winner` reached the target; robots stay where they ended.
    Solved { winner: PlayerId, moves: u8 },
    /// Nobody solved it; robots are back where the round started.
    Unsolved,
}

/// Why an action was refused. The round is unchanged when this is returned.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoundError {
    /// The action doesn't fit the current phase.
    WrongPhase,
    /// The bidding window has passed.
    BiddingClosed,
    /// The bidding window is still open.
    BiddingStillOpen,
    /// Bidding can't close without bids; abandon the round instead.
    NoBids,
    /// Only the current demonstrator may act.
    NotYourTurn,
    /// The robot can't move that way; such moves don't count.
    RobotDidNotMove,
    Bid(BidError),
}

impl fmt::Display for RoundError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Bid(e) => e.fmt(f),
            other => write!(f, "{other:?}"),
        }
    }
}

impl std::error::Error for RoundError {}

impl From<BidError> for RoundError {
    fn from(e: BidError) -> Self {
        Self::Bid(e)
    }
}

/// A round on a given target. Methods that need the board take it as an
/// argument so the round owns no references.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Round {
    target: Target,
    window_ms: u64,
    start: Robots,
    robots: Robots,
    bids: Bids,
    /// Bidders still to demonstrate, best first; the current one is first.
    queue: Vec<Bid>,
    moves_made: u8,
    outcome: Option<Phase>,
}

impl Round {
    /// Starts bidding on `target` with robots at `robots`. The window closes
    /// `window_ms` after the first bid.
    #[must_use]
    pub fn new(target: Target, robots: Robots, window_ms: u64) -> Self {
        Self {
            target,
            window_ms,
            start: robots,
            robots,
            bids: Bids::new(),
            queue: Vec::new(),
            moves_made: 0,
            outcome: None,
        }
    }

    #[must_use]
    pub const fn target(&self) -> Target {
        self.target
    }

    /// Current robot positions, including a demonstration in progress.
    #[must_use]
    pub const fn robots(&self) -> &Robots {
        &self.robots
    }

    #[must_use]
    pub fn phase(&self) -> Phase {
        if let Some(outcome) = self.outcome {
            return outcome;
        }
        match self.queue.first() {
            Some(bid) => Phase::Demonstrating {
                player: bid.player,
                bid: bid.moves,
                moves_made: self.moves_made,
            },
            None => Phase::Bidding,
        }
    }

    /// Current bids, best first (fewest moves, then earliest).
    #[must_use]
    pub fn bids(&self) -> Vec<Bid> {
        self.bids.ranking()
    }

    /// When bidding ends, once someone has bid.
    #[must_use]
    pub fn bidding_closes_at(&self) -> Option<Timestamp> {
        self.bids
            .first_bid_at()
            .map(|t| Timestamp(t.0.saturating_add(self.window_ms)))
    }

    /// Records a bid made at `at`.
    ///
    /// # Errors
    /// Outside bidding, after the window, or if [`Bids::place`] refuses it.
    pub fn bid(&mut self, player: PlayerId, moves: u8, at: Timestamp) -> Result<(), RoundError> {
        if self.phase() != Phase::Bidding {
            return Err(RoundError::WrongPhase);
        }
        if self.bidding_closes_at().is_some_and(|end| at >= end) {
            return Err(RoundError::BiddingClosed);
        }
        Ok(self.bids.place(player, moves, at)?)
    }

    /// Ends bidding at `now` and hands the board to the best bidder.
    ///
    /// # Errors
    /// Outside bidding, with no bids, or before the window has passed.
    pub fn close_bidding(&mut self, now: Timestamp) -> Result<(), RoundError> {
        if self.phase() != Phase::Bidding {
            return Err(RoundError::WrongPhase);
        }
        let end = self.bidding_closes_at().ok_or(RoundError::NoBids)?;
        if now < end {
            return Err(RoundError::BiddingStillOpen);
        }
        self.queue = self.bids.ranking();
        Ok(())
    }

    /// Ends a round nobody bid on.
    ///
    /// # Errors
    /// Outside bidding, or once someone has bid.
    pub fn abandon(&mut self) -> Result<(), RoundError> {
        if self.phase() != Phase::Bidding || !self.bids.is_empty() {
            return Err(RoundError::WrongPhase);
        }
        self.outcome = Some(Phase::Unsolved);
        Ok(())
    }

    /// The demonstrator moves `robot`. Solves the round as soon as a robot
    /// that may complete the target stands on it; uses up the bid otherwise.
    ///
    /// # Errors
    /// If it isn't `player`'s demonstration or the robot can't move.
    pub fn move_robot(
        &mut self,
        board: &Board,
        player: PlayerId,
        robot: RobotColor,
        dir: Direction,
    ) -> Result<(), RoundError> {
        let bid = self.current_demonstrator(player)?;
        let moved = board.move_robot(&self.robots, robot, dir);
        if moved == self.robots {
            return Err(RoundError::RobotDidNotMove);
        }
        self.robots = moved;
        self.moves_made += 1;
        if self.target_reached(board) {
            self.outcome = Some(Phase::Solved {
                winner: player,
                moves: self.moves_made,
            });
        } else if self.moves_made >= bid.moves {
            self.next_demonstrator();
        }
        Ok(())
    }

    /// The demonstrator gives up (or the server times them out).
    ///
    /// # Errors
    /// If it isn't `player`'s demonstration.
    pub fn concede(&mut self, player: PlayerId) -> Result<(), RoundError> {
        self.current_demonstrator(player)?;
        self.next_demonstrator();
        Ok(())
    }

    fn current_demonstrator(&self, player: PlayerId) -> Result<Bid, RoundError> {
        match self.phase() {
            Phase::Demonstrating { .. } => {}
            _ => return Err(RoundError::WrongPhase),
        }
        let bid = self.queue[0];
        if bid.player == player {
            Ok(bid)
        } else {
            Err(RoundError::NotYourTurn)
        }
    }

    /// Resets the robots and passes to the next bidder, if any.
    fn next_demonstrator(&mut self) {
        self.robots = self.start;
        self.moves_made = 0;
        self.queue.remove(0);
        if self.queue.is_empty() {
            self.outcome = Some(Phase::Unsolved);
        }
    }

    fn target_reached(&self, board: &Board) -> bool {
        completes(board, self.target, self.robots)
    }
}

/// Whether a robot that may complete `target` stands on its cell.
pub(crate) fn completes(board: &Board, target: Target, robots: Robots) -> bool {
    board
        .position_of(target)
        .and_then(|cell| robots.robot_at(cell))
        .is_some_and(|robot| target.is_reached_by(robot))
}
