use RobotColor::{Blue, Green, Red, Yellow};
use Symbol::{Gear, Moon, Planet, Star};
use domain::{BOARD_SIZE, Direction, Position, RobotColor, Symbol, Target, fixed_board};

fn pos(row: u8, col: u8) -> Position {
    Position::new(row, col).unwrap()
}

#[test]
fn has_every_target_exactly_once() {
    let board = fixed_board::board();
    assert_eq!(board.targets().len(), 17);
    for target in Target::all() {
        assert!(board.position_of(target).is_some(), "missing {target:?}");
    }
}

#[test]
fn blocks_exactly_the_centre() {
    let board = fixed_board::board();
    let blocked: Vec<_> = Position::all().filter(|p| board.is_blocked(*p)).collect();
    assert_eq!(blocked, [pos(7, 7), pos(7, 8), pos(8, 7), pos(8, 8)]);
}

#[test]
fn robots_start_on_free_cells_off_targets() {
    let board = fixed_board::board();
    let robots = fixed_board::starting_robots();
    for robot in RobotColor::ALL {
        let at = robots.position(robot);
        assert!(!board.is_blocked(at), "{robot:?} starts on a blocked cell");
        assert_eq!(board.target_at(at), None, "{robot:?} starts on a target");
    }
}

#[test]
fn outer_edge_is_walled_like_the_legacy_data() {
    let board = fixed_board::board();
    let last = BOARD_SIZE - 1;
    for i in 0..BOARD_SIZE {
        assert!(board.has_wall(pos(0, i), Direction::Up));
        assert!(board.has_wall(pos(last, i), Direction::Down));
        assert!(board.has_wall(pos(i, 0), Direction::Left));
        assert!(board.has_wall(pos(i, last), Direction::Right));
    }
}

/// Expected stops from the start position, computed independently by
/// simulating the legacy JavaScript movement on the legacy PHP data
/// (`legacy/tools/board_oracle.py`).
#[test]
fn first_moves_match_the_legacy_game() {
    let board = fixed_board::board();
    let robots = fixed_board::starting_robots();
    #[rustfmt::skip]
    let expected = [
        (Red, Direction::Up, pos(0, 1)),
        (Red, Direction::Down, pos(1, 1)),
        (Red, Direction::Left, pos(1, 0)),
        (Red, Direction::Right, pos(1, 3)),
        (Yellow, Direction::Up, pos(0, 0)),
        (Yellow, Direction::Down, pos(5, 0)),
        (Yellow, Direction::Left, pos(4, 0)),
        (Yellow, Direction::Right, pos(4, 15)),
        (Green, Direction::Up, pos(12, 0)),
        (Green, Direction::Down, pos(15, 0)),
        (Green, Direction::Left, pos(14, 0)),
        (Green, Direction::Right, pos(14, 2)),
        (Blue, Direction::Up, pos(10, 13)),
        (Blue, Direction::Down, pos(15, 13)),
        (Blue, Direction::Left, pos(15, 12)),
        (Blue, Direction::Right, pos(15, 15)),
    ];
    for (robot, dir, stop) in expected {
        assert_eq!(board.slide(&robots, robot, dir), stop, "{robot:?} {dir:?}");
    }
}

/// Every target sits on its legacy cell. Expected values from
/// `legacy/tools/board_oracle.py`.
#[test]
fn targets_sit_on_their_legacy_cells() {
    let board = fixed_board::board();
    #[rustfmt::skip]
    let expected = [
        (pos(1, 4), Target::Colored(Red, Moon)),
        (pos(1, 14), Target::Colored(Red, Gear)),
        (pos(2, 1), Target::Colored(Green, Gear)),
        (pos(2, 11), Target::Colored(Blue, Star)),
        (pos(3, 6), Target::Colored(Yellow, Star)),
        (pos(6, 3), Target::Colored(Blue, Planet)),
        (pos(6, 13), Target::Colored(Green, Moon)),
        (pos(7, 10), Target::Colored(Yellow, Planet)),
        (pos(8, 5), Target::Vortex),
        (pos(9, 1), Target::Colored(Blue, Moon)),
        (pos(9, 13), Target::Colored(Blue, Gear)),
        (pos(10, 4), Target::Colored(Green, Planet)),
        (pos(11, 9), Target::Colored(Yellow, Moon)),
        (pos(13, 5), Target::Colored(Red, Star)),
        (pos(13, 14), Target::Colored(Red, Planet)),
        (pos(14, 3), Target::Colored(Yellow, Gear)),
        (pos(14, 10), Target::Colored(Green, Star)),
    ];
    for (cell, target) in expected {
        assert_eq!(board.position_of(target), Some(cell), "{target:?}");
    }
}

/// One move: which robot, which way.
type Move = (RobotColor, Direction);

/// Twelve-move games from the start position end where the legacy game
/// ends them, exercising interior walls and robots blocking each other.
/// Expected values from `legacy/tools/board_oracle.py` (fixed seed).
#[test]
fn move_sequences_match_the_legacy_game() {
    use Direction::{Down, Left, Right, Up};
    let board = fixed_board::board();
    #[rustfmt::skip]
    let cases: [(&[Move], [Position; 4]); 8] = [
        (&[(Blue, Right), (Blue, Left), (Yellow, Up), (Yellow, Left), (Blue, Down), (Blue, Right), (Blue, Up), (Blue, Up), (Red, Left), (Yellow, Right), (Red, Right), (Yellow, Down)], [pos(1, 3), pos(12, 15), pos(14, 0), pos(1, 1)]),
        (&[(Red, Down), (Red, Right), (Yellow, Left), (Red, Down), (Red, Right), (Yellow, Up), (Blue, Up), (Yellow, Down), (Yellow, Up), (Yellow, Right), (Yellow, Up), (Blue, Left)], [pos(6, 13), pos(10, 4), pos(14, 0), pos(0, 1)]),
        (&[(Green, Down), (Blue, Up), (Yellow, Left), (Yellow, Left), (Blue, Up), (Yellow, Right), (Yellow, Right), (Yellow, Up), (Yellow, Left), (Red, Right), (Blue, Down), (Green, Up)], [pos(1, 3), pos(15, 13), pos(12, 0), pos(4, 0)]),
        (&[(Yellow, Right), (Green, Right), (Yellow, Up), (Yellow, Up), (Blue, Right), (Red, Up), (Blue, Up), (Red, Up), (Green, Up), (Yellow, Down), (Red, Up), (Red, Down)], [pos(1, 1), pos(12, 15), pos(0, 2), pos(11, 15)]),
        (&[(Green, Up), (Green, Up), (Blue, Down), (Blue, Up), (Blue, Left), (Green, Down), (Blue, Right), (Red, Left), (Blue, Right), (Yellow, Left), (Green, Up), (Green, Right)], [pos(1, 0), pos(10, 15), pos(12, 15), pos(4, 0)]),
        (&[(Blue, Up), (Green, Left), (Red, Up), (Yellow, Left), (Green, Up), (Blue, Down), (Yellow, Down), (Blue, Right), (Red, Down), (Blue, Up), (Red, Right), (Yellow, Right)], [pos(1, 3), pos(12, 15), pos(12, 0), pos(5, 15)]),
        (&[(Blue, Up), (Yellow, Left), (Green, Left), (Yellow, Right), (Yellow, Left), (Blue, Up), (Blue, Left), (Green, Down), (Blue, Up), (Green, Up), (Blue, Left), (Green, Left)], [pos(1, 1), pos(1, 4), pos(12, 0), pos(4, 0)]),
        (&[(Green, Left), (Yellow, Right), (Yellow, Up), (Blue, Up), (Red, Left), (Red, Left), (Red, Up), (Yellow, Up), (Red, Up), (Green, Right), (Yellow, Up), (Blue, Right)], [pos(0, 0), pos(10, 15), pos(14, 2), pos(4, 15)]),
    ];
    for (moves, expected) in cases {
        let end = moves
            .iter()
            .fold(fixed_board::starting_robots(), |robots, &(robot, dir)| {
                board.move_robot(&robots, robot, dir)
            });
        let actual = RobotColor::ALL.map(|robot| end.position(robot));
        assert_eq!(actual, expected, "after {moves:?}");
    }
}
