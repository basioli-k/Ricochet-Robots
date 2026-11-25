//! The JSON wire format, as clients see it.

use domain::{BoardBuilder, Direction, PlayerId, Position, Target};
use serde_json::json;
use server::protocol::{BoardView, Dir, parse_player_id, player_id};

const ANA: PlayerId = PlayerId::new(0xa1);

#[test]
fn player_ids_are_32_hex_digits_both_ways() {
    let text = player_id(ANA);
    assert_eq!(text, "000000000000000000000000000000a1");
    assert_eq!(parse_player_id(&text), Some(ANA));
    for bad in ["a1", "zz000000000000000000000000000000", ""] {
        assert_eq!(parse_player_id(bad), None, "{bad}");
    }
}

#[test]
fn walls_are_listed_from_each_side() {
    let pos = Position::new(3, 4).unwrap();
    let board = BoardBuilder::new()
        .wall(pos, Direction::Right)
        .target(Position::new(0, 0).unwrap(), Target::Vortex)
        .unwrap()
        .build()
        .unwrap();
    let view = BoardView::from(&board);
    let at = |row, col, side| {
        view.walls
            .iter()
            .any(|w| (w.pos.row, w.pos.col, w.side) == (row, col, side))
    };
    assert!(at(3, 4, Dir::Right) && at(3, 5, Dir::Left));
    assert!(
        !at(0, 0, Dir::Up),
        "the edge is implied by the board's size"
    );
    assert_eq!(
        serde_json::to_value(view.targets[0]).unwrap(),
        json!({ "pos": { "row": 0, "col": 0 }, "target": { "kind": "vortex" } })
    );
}
