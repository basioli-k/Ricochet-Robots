use RobotColor::{Blue, Green, Red, Yellow};
use domain::{BOARD_SIZE, Direction, Position, RobotColor, Target, fixed_board};

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
/// simulating the legacy JavaScript movement on the legacy PHP data.
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
