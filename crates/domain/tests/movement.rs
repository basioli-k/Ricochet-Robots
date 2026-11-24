use RobotColor::{Blue, Green, Red, Yellow};
use domain::{
    BOARD_SIZE, Board, BoardBuilder, Direction, Position, RobotColor, Robots, SharedCell,
};
use proptest::prelude::*;

fn pos(row: u8, col: u8) -> Position {
    Position::new(row, col).unwrap()
}

/// Red at (5,5), the others parked in corners out of the way.
fn red_at_centre() -> Robots {
    Robots::new([pos(5, 5), pos(0, 0), pos(0, 15), pos(15, 0)]).unwrap()
}

#[test]
fn rejects_two_robots_on_one_cell() {
    let err = Robots::new([pos(1, 1), pos(2, 2), pos(1, 1), pos(3, 3)]).unwrap_err();
    assert_eq!(err, SharedCell(pos(1, 1)));
}

#[test]
fn slides_to_the_edge_on_an_empty_board() {
    let board = BoardBuilder::new().build().unwrap();
    let robots = red_at_centre();
    assert_eq!(board.slide(&robots, Red, Direction::Up), pos(0, 5));
    assert_eq!(board.slide(&robots, Red, Direction::Down), pos(15, 5));
    assert_eq!(board.slide(&robots, Red, Direction::Left), pos(5, 0));
    assert_eq!(board.slide(&robots, Red, Direction::Right), pos(5, 15));
}

#[test]
fn stops_at_a_wall_seen_from_either_side() {
    let board = BoardBuilder::new()
        .wall(pos(5, 9), Direction::Right) // stops a robot moving right
        .wall(pos(5, 2), Direction::Right) // stops a robot moving left at (5,3)
        .build()
        .unwrap();
    let robots = red_at_centre();
    assert_eq!(board.slide(&robots, Red, Direction::Right), pos(5, 9));
    assert_eq!(board.slide(&robots, Red, Direction::Left), pos(5, 3));
}

#[test]
fn stops_before_another_robot() {
    let board = BoardBuilder::new().build().unwrap();
    let robots = Robots::new([pos(5, 5), pos(5, 12), pos(0, 15), pos(15, 0)]).unwrap();
    assert_eq!(board.slide(&robots, Red, Direction::Right), pos(5, 11));
}

#[test]
fn stops_before_a_blocked_cell() {
    let board = BoardBuilder::new().block(pos(9, 5)).build().unwrap();
    assert_eq!(
        board.slide(&red_at_centre(), Red, Direction::Down),
        pos(8, 5)
    );
}

#[test]
fn does_not_move_when_blocked_immediately() {
    let board = BoardBuilder::new()
        .wall(pos(5, 5), Direction::Up)
        .build()
        .unwrap();
    let robots = red_at_centre();
    assert_eq!(board.slide(&robots, Red, Direction::Up), pos(5, 5));
    assert_eq!(board.move_robot(&robots, Red, Direction::Up), robots);
}

#[test]
fn move_robot_moves_only_that_robot() {
    let board = BoardBuilder::new().build().unwrap();
    let before = red_at_centre();
    let after = board.move_robot(&before, Red, Direction::Down);
    assert_eq!(after.position(Red), pos(15, 5));
    for other in [Blue, Green, Yellow] {
        assert_eq!(after.position(other), before.position(other));
    }
}

fn any_position() -> impl Strategy<Value = Position> {
    (0..BOARD_SIZE, 0..BOARD_SIZE).prop_map(|(r, c)| pos(r, c))
}

fn any_direction() -> impl Strategy<Value = Direction> {
    prop::sample::select(Direction::ALL.to_vec())
}

fn any_robot() -> impl Strategy<Value = RobotColor> {
    prop::sample::select(RobotColor::ALL.to_vec())
}

/// A random board with walls and blocked cells, and robots off blocked cells.
fn any_setup() -> impl Strategy<Value = (Board, Robots)> {
    (
        prop::collection::vec((any_position(), any_direction()), 0..80),
        prop::collection::vec(any_position(), 0..6),
        prop::collection::hash_set(any_position(), 4),
    )
        .prop_filter(
            "robots must not stand on blocked cells",
            |(_, blocked, robots)| robots.iter().all(|r| !blocked.contains(r)),
        )
        .prop_map(|(walls, blocked, robots)| {
            let builder = walls
                .into_iter()
                .fold(BoardBuilder::new(), |b, (p, d)| b.wall(p, d));
            let board = blocked
                .into_iter()
                .fold(builder, BoardBuilder::block)
                .build()
                .unwrap();
            let cells: Vec<_> = robots.into_iter().collect();
            (
                board,
                Robots::new([cells[0], cells[1], cells[2], cells[3]]).unwrap(),
            )
        })
}

/// Whether a robot at `from` may enter the next cell in `dir`.
fn can_enter(board: &Board, robots: Robots, from: Position, dir: Direction) -> bool {
    !board.has_wall(from, dir)
        && from
            .step(dir)
            .is_some_and(|n| !board.is_blocked(n) && robots.robot_at(n).is_none())
}

proptest! {
    /// The robot passes only free cells, never through a wall, and stops at
    /// the first obstacle rather than earlier.
    #[test]
    fn slides_through_free_cells_to_the_first_obstacle(
        (board, robots) in any_setup(), robot in any_robot(), dir in any_direction(),
    ) {
        let end = board.slide(&robots, robot, dir);
        let mut at = robots.position(robot);
        while at != end {
            prop_assert!(can_enter(&board, robots, at, dir), "crossed an obstacle at {:?}", at);
            at = at.step(dir).unwrap();
        }
        prop_assert!(!can_enter(&board, robots, end, dir), "stopped early at {:?}", end);
    }

    /// Sliding again in the same direction is a no-op.
    #[test]
    fn sliding_twice_is_a_no_op(
        (board, robots) in any_setup(), robot in any_robot(), dir in any_direction(),
    ) {
        let once = board.move_robot(&robots, robot, dir);
        prop_assert_eq!(board.move_robot(&once, robot, dir), once);
    }
}
