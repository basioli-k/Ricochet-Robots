//! Checks a board and start position are fit to play.

use crate::board::Board;
use crate::geometry::Position;
use crate::pieces::{RobotColor, Target};
use crate::robots::Robots;
use crate::solver::solve;

/// The 2x2 centre block every board must have.
const CENTRE: [Position; 4] = [cell(7, 7), cell(7, 8), cell(8, 7), cell(8, 8)];

/// Evaluated at compile time, so an out-of-range cell fails the build.
const fn cell(row: u8, col: u8) -> Position {
    match Position::new(row, col) {
        Some(p) => p,
        None => panic!("centre cell off the board"),
    }
}

/// One reason a board is not fit to play.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoardProblem {
    CentreNotBlocked(Position),
    MissingTarget(Target),
    RobotOnBlockedCell(RobotColor),
    /// A robot on a target would let a round be won in zero moves.
    RobotOnTarget(RobotColor),
    /// No solution from the start position within the move limit.
    Unreachable(Target),
}

/// Checks what board construction can't guarantee. The builder already
/// makes walls two-sided and targets one per cell, and the edge always
/// stops movement, so those are not re-checked.
#[derive(Debug, Clone, Copy)]
pub struct BoardValidator {
    max_moves: u8,
}

impl BoardValidator {
    /// A target counts as reachable if any robots can complete it from the
    /// start position in at most `max_moves` moves.
    #[must_use]
    pub const fn new(max_moves: u8) -> Self {
        Self { max_moves }
    }

    /// Every problem found, in a stable order: centre, targets, robots,
    /// reachability.
    ///
    /// # Errors
    /// The list of problems, if there is any.
    pub fn validate(&self, board: &Board, robots: Robots) -> Result<(), Vec<BoardProblem>> {
        let mut problems = Vec::new();
        for cell in CENTRE {
            if !board.is_blocked(cell) {
                problems.push(BoardProblem::CentreNotBlocked(cell));
            }
        }
        let placed: Vec<Target> = Target::all()
            .filter(|t| board.position_of(*t).is_some())
            .collect();
        problems.extend(
            Target::all()
                .filter(|t| !placed.contains(t))
                .map(BoardProblem::MissingTarget),
        );
        for robot in RobotColor::ALL {
            let cell = robots.position(robot);
            if board.is_blocked(cell) {
                problems.push(BoardProblem::RobotOnBlockedCell(robot));
            }
            if board.target_at(cell).is_some() {
                problems.push(BoardProblem::RobotOnTarget(robot));
            }
        }
        for target in placed {
            if solve(board, robots, target, &RobotColor::ALL, self.max_moves).is_none() {
                problems.push(BoardProblem::Unreachable(target));
            }
        }
        if problems.is_empty() {
            Ok(())
        } else {
            Err(problems)
        }
    }
}
