//! Robot movement: a robot slides until something stops it.

use crate::board::Board;
use crate::geometry::{Direction, Position};
use crate::pieces::RobotColor;
use crate::robots::Robots;

impl Board {
    /// Where `robot` stops when moved in `dir`: just before the first wall,
    /// board edge, blocked cell or other robot. Returns its current cell if
    /// it can't move at all. This is the only movement rule in the game.
    #[must_use]
    pub fn slide(&self, robots: &Robots, robot: RobotColor, dir: Direction) -> Position {
        let mut pos = robots.position(robot);
        while !self.has_wall(pos, dir) {
            let Some(next) = pos.step(dir) else { break };
            if self.is_blocked(next) || robots.robot_at(next).is_some() {
                break;
            }
            pos = next;
        }
        pos
    }

    /// The robots after moving `robot` in `dir`; unchanged if it can't move.
    #[must_use]
    pub fn move_robot(&self, robots: &Robots, robot: RobotColor, dir: Direction) -> Robots {
        robots.moved(robot, self.slide(robots, robot, dir))
    }
}
