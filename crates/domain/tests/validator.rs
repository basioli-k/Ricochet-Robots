use RobotColor::{Blue, Green, Red, Yellow};
use domain::{
    Board, BoardBuilder, BoardProblem, BoardValidator, Direction, Position, RobotColor, Robots,
    Symbol, Target, fixed_board,
};

/// Generous for the fixed board, whose hardest target needs 10 moves.
const LIMIT: u8 = 12;

fn pos(row: u8, col: u8) -> Position {
    Position::new(row, col).unwrap()
}

fn validate(board: &Board, robots: Robots) -> Vec<BoardProblem> {
    BoardValidator::new(LIMIT)
        .validate(board, robots)
        .err()
        .unwrap_or_default()
}

#[test]
fn the_fixed_board_is_fit_to_play() {
    assert_eq!(
        validate(&fixed_board::board(), fixed_board::starting_robots()),
        []
    );
}

#[test]
fn reports_an_unblocked_centre_and_missing_targets() {
    let board = BoardBuilder::new().build().unwrap();
    let robots = Robots::new([pos(0, 0), pos(0, 15), pos(15, 0), pos(15, 15)]).unwrap();
    let problems = validate(&board, robots);
    assert_eq!(problems[0], BoardProblem::CentreNotBlocked(pos(7, 7)));
    assert_eq!(problems.len(), 4 + 17);
    assert!(problems.contains(&BoardProblem::MissingTarget(Target::Vortex)));
}

#[test]
fn reports_robots_on_blocked_cells_or_targets() {
    let red_planet = fixed_board::board()
        .position_of(Target::Colored(Red, Symbol::Planet))
        .unwrap();
    let vortex = fixed_board::board().position_of(Target::Vortex).unwrap();
    let robots = Robots::new([pos(7, 7), red_planet, pos(14, 0), vortex]).unwrap();
    let problems = validate(&fixed_board::board(), robots);
    assert!(problems.contains(&BoardProblem::RobotOnBlockedCell(Red)));
    assert!(problems.contains(&BoardProblem::RobotOnTarget(Blue)));
    assert!(problems.contains(&BoardProblem::RobotOnTarget(Yellow)));
    assert!(!problems.contains(&BoardProblem::RobotOnBlockedCell(Green)));
    assert!(!problems.contains(&BoardProblem::RobotOnTarget(Green)));
}

/// The limit is inclusive: the fixed board's hardest targets (the vortex
/// and Blue Gear) need exactly 10 moves.
#[test]
fn the_move_limit_is_inclusive() {
    let (board, robots) = (fixed_board::board(), fixed_board::starting_robots());
    assert_eq!(BoardValidator::new(10).validate(&board, robots), Ok(()));
    let mut problems = BoardValidator::new(9).validate(&board, robots).unwrap_err();
    problems.sort_by_key(|p| format!("{p:?}"));
    assert_eq!(
        problems,
        [
            BoardProblem::Unreachable(Target::Colored(Blue, Symbol::Gear)),
            BoardProblem::Unreachable(Target::Vortex),
        ]
    );
}

#[test]
fn reports_a_walled_off_target_as_unreachable() {
    // Box the red moon in: walls on all four sides of its cell.
    let fixed = fixed_board::board();
    let moon = Target::Colored(Red, Symbol::Moon);
    let cell = fixed.position_of(moon).unwrap();
    let mut builder = BoardBuilder::new();
    for p in Position::all() {
        for dir in Direction::ALL {
            if fixed.has_wall(p, dir) || p == cell {
                builder = builder.wall(p, dir);
            }
        }
        if fixed.is_blocked(p) {
            builder = builder.block(p);
        }
    }
    for &(p, t) in fixed.targets() {
        builder = builder.target(p, t).unwrap();
    }
    let boxed = builder.build().unwrap();
    assert_eq!(
        validate(&boxed, fixed_board::starting_robots()),
        [BoardProblem::Unreachable(moon)]
    );
}
