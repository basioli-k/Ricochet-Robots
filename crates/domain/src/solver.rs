//! Shortest solutions by breadth-first search over robot positions.

use std::collections::{HashMap, VecDeque};

use crate::board::Board;
use crate::geometry::Direction;
use crate::pieces::{RobotColor, Target};
use crate::robots::Robots;
use crate::round::completes;

/// One move: which robot, which way.
pub type Move = (RobotColor, Direction);

/// The shortest move sequence after which `target` is completed, moving only
/// the robots in `movers`, at most `max_moves` long. Empty if it is already
/// completed; `None` if there is no solution within the limit.
///
/// Cost grows with the number of movers: one robot has at most 256 states,
/// all four can have millions, so keep `max_moves` small when moving several.
#[must_use]
pub fn solve(
    board: &Board,
    robots: Robots,
    target: Target,
    movers: &[RobotColor],
    max_moves: u8,
) -> Option<Vec<Move>> {
    if completes(board, target, robots) {
        return Some(Vec::new());
    }
    // How each state was first reached: (previous state, move, depth).
    let mut came_from: HashMap<Robots, (Robots, Move, u8)> = HashMap::new();
    let mut frontier = VecDeque::from([(robots, 0u8)]);
    while let Some((state, depth)) = frontier.pop_front() {
        if depth == max_moves {
            continue;
        }
        for &robot in movers {
            for dir in Direction::ALL {
                let next = board.move_robot(&state, robot, dir);
                // Skip non-moves and states already seen; the start has no
                // `came_from` entry, so it is checked separately.
                if next == state || next == robots || came_from.contains_key(&next) {
                    continue;
                }
                came_from.insert(next, (state, (robot, dir), depth + 1));
                if completes(board, target, next) {
                    return Some(path_to(next, robots, &came_from));
                }
                frontier.push_back((next, depth + 1));
            }
        }
    }
    None
}

fn path_to(
    end: Robots,
    start: Robots,
    came_from: &HashMap<Robots, (Robots, Move, u8)>,
) -> Vec<Move> {
    let mut moves = Vec::new();
    let mut state = end;
    while state != start {
        let (previous, step, _) = came_from[&state];
        moves.push(step);
        state = previous;
    }
    moves.reverse();
    moves
}
