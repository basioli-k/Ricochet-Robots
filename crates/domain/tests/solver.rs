use Direction::{Right, Up};
use RobotColor::{Blue, Red};
use domain::{Board, BoardBuilder, Direction, Position, RobotColor, Robots, Symbol, Target, solve};

const RED_MOON: Target = Target::Colored(Red, Symbol::Moon);

fn pos(row: u8, col: u8) -> Position {
    Position::new(row, col).unwrap()
}

/// Red Moon at (3,5), in Red's column. Red alone slides past it; Blue can
/// stop at (2,5) thanks to a wall on its right and act as a blocker.
fn board() -> Board {
    BoardBuilder::new()
        .wall(pos(2, 5), Right)
        .target(pos(3, 5), RED_MOON)
        .unwrap()
        .build()
        .unwrap()
}

fn robots() -> Robots {
    Robots::new([pos(5, 5), pos(2, 0), pos(15, 0), pos(15, 15)]).unwrap()
}

fn replay(board: &Board, robots: Robots, moves: &[(RobotColor, Direction)]) -> Robots {
    moves
        .iter()
        .fold(robots, |r, &(robot, dir)| board.move_robot(&r, robot, dir))
}

#[test]
fn an_already_completed_target_needs_no_moves() {
    let on_target = Robots::new([pos(3, 5), pos(2, 0), pos(15, 0), pos(15, 15)]).unwrap();
    assert_eq!(
        solve(&board(), on_target, RED_MOON, &RobotColor::ALL, 5),
        Some(vec![])
    );
}

#[test]
fn finds_the_shortest_solution_using_a_blocker() {
    let moves = solve(&board(), robots(), RED_MOON, &RobotColor::ALL, 5).unwrap();
    assert_eq!(moves, [(Blue, Right), (Red, Up)]);
    let end = replay(&board(), robots(), &moves);
    assert_eq!(end.position(Red), pos(3, 5));
}

#[test]
fn only_the_given_robots_move() {
    assert_eq!(solve(&board(), robots(), RED_MOON, &[Red], 10), None);
}

#[test]
fn respects_the_move_limit() {
    assert_eq!(
        solve(&board(), robots(), RED_MOON, &RobotColor::ALL, 1),
        None
    );
    assert!(solve(&board(), robots(), RED_MOON, &RobotColor::ALL, 2).is_some());
}

#[test]
fn a_target_missing_from_the_board_has_no_solution() {
    let empty = BoardBuilder::new().build().unwrap();
    assert_eq!(solve(&empty, robots(), RED_MOON, &RobotColor::ALL, 3), None);
}
