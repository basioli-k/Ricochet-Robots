use domain::{BoardBuilder, BoardError, Direction, Position, RobotColor, Symbol, Target};
use proptest::prelude::*;

fn pos(row: u8, col: u8) -> Position {
    Position::new(row, col).unwrap()
}

const RED_MOON: Target = Target::Colored(RobotColor::Red, Symbol::Moon);

#[test]
fn a_wall_is_seen_from_both_cells() {
    let board = BoardBuilder::new()
        .wall(pos(3, 3), Direction::Right)
        .build()
        .unwrap();
    assert!(board.has_wall(pos(3, 3), Direction::Right));
    assert!(board.has_wall(pos(3, 4), Direction::Left));
    assert!(!board.has_wall(pos(3, 3), Direction::Left));
}

#[test]
fn a_wall_on_the_edge_is_harmless() {
    let board = BoardBuilder::new()
        .wall(pos(0, 0), Direction::Up)
        .build()
        .unwrap();
    assert!(board.has_wall(pos(0, 0), Direction::Up));
}

#[test]
fn finds_targets_by_cell_and_by_target() {
    let board = BoardBuilder::new()
        .target(pos(1, 4), RED_MOON)
        .unwrap()
        .build()
        .unwrap();
    assert_eq!(board.target_at(pos(1, 4)), Some(RED_MOON));
    assert_eq!(board.position_of(RED_MOON), Some(pos(1, 4)));
    assert_eq!(board.target_at(pos(1, 5)), None);
}

#[test]
fn rejects_two_targets_on_one_cell() {
    let result = BoardBuilder::new()
        .target(pos(1, 4), RED_MOON)
        .and_then(|b| b.target(pos(1, 4), Target::Vortex));
    assert_eq!(result.unwrap_err(), BoardError::CellHasTarget(pos(1, 4)));
}

#[test]
fn rejects_a_target_placed_twice() {
    let result = BoardBuilder::new()
        .target(pos(1, 4), RED_MOON)
        .and_then(|b| b.target(pos(2, 2), RED_MOON));
    assert_eq!(result.unwrap_err(), BoardError::TargetPlacedTwice(RED_MOON));
}

#[test]
fn rejects_a_target_on_a_blocked_cell() {
    let result = BoardBuilder::new()
        .block(pos(7, 7))
        .target(pos(7, 7), Target::Vortex)
        .unwrap()
        .build();
    assert_eq!(
        result.unwrap_err(),
        BoardError::TargetOnBlockedCell(pos(7, 7))
    );
}

fn any_position() -> impl Strategy<Value = Position> {
    (0..domain::BOARD_SIZE, 0..domain::BOARD_SIZE).prop_map(|(r, c)| pos(r, c))
}

fn any_direction() -> impl Strategy<Value = Direction> {
    prop::sample::select(Direction::ALL.to_vec())
}

proptest! {
    /// The board has exactly the added walls, each seen from both sides:
    /// none dropped, none invented.
    #[test]
    fn walls_are_exactly_the_added_ones_from_both_sides(
        walls in prop::collection::vec((any_position(), any_direction()), 0..60),
    ) {
        let board = walls
            .iter()
            .fold(BoardBuilder::new(), |b, &(p, d)| b.wall(p, d))
            .build()
            .unwrap();
        let added = |p: Position, d: Direction| {
            walls.contains(&(p, d)) || p.step(d).is_some_and(|n| walls.contains(&(n, d.opposite())))
        };
        for p in Position::all() {
            for d in Direction::ALL {
                prop_assert_eq!(board.has_wall(p, d), added(p, d), "{:?} {:?}", p, d);
            }
        }
    }
}
