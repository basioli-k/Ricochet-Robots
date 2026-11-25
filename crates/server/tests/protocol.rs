//! The JSON wire format, as clients see it.

use domain::{
    BoardBuilder, Direction, GameError, Phase, PlayerId, Position, RobotColor, RoundError, Target,
    Timestamp, fixed_board,
};
use serde_json::json;
use server::protocol::{
    BoardView, ClientMessage, Dir, Robot, ServerMessage, parse_player_id, player_id,
    room_error_code,
};
use server::rooms::state::{Member, RoomError, RoomEvent, RoomState};

const ANA: PlayerId = PlayerId::new(0xa1);

fn json_of(message: ServerMessage) -> serde_json::Value {
    serde_json::to_value(message).unwrap()
}

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
fn client_messages_are_tagged_by_type() {
    let parse = |v: serde_json::Value| serde_json::from_value::<ClientMessage>(v);
    assert_eq!(
        parse(json!({ "type": "move", "robot": "red", "dir": "up" })).unwrap(),
        ClientMessage::Move {
            robot: Robot::Red,
            dir: Dir::Up
        }
    );
    assert_eq!(
        parse(json!({ "type": "bid", "moves": 7 })).unwrap(),
        ClientMessage::Bid { moves: 7 }
    );
    assert_eq!(
        parse(json!({ "type": "start" })).unwrap(),
        ClientMessage::Start
    );
    for bad in [
        json!({ "type": "bid", "moves": 300 }),
        json!({ "type": "fly" }),
        json!({ "moves": 3 }),
    ] {
        assert!(parse(bad.clone()).is_err(), "{bad}");
    }
}

#[test]
fn room_events_become_server_messages() {
    assert_eq!(
        json_of(
            RoomEvent::BidPlaced {
                player: ANA,
                moves: 4,
                closes_at: Timestamp(61_000)
            }
            .into()
        ),
        json!({
            "type": "bid_placed",
            "player": player_id(ANA),
            "moves": 4,
            "closes_at": 61_000
        })
    );
    assert_eq!(
        json_of(
            RoomEvent::RoundEnded {
                phase: Phase::Solved {
                    winner: ANA,
                    moves: 2
                }
            }
            .into()
        ),
        json!({
            "type": "round_resolved",
            "phase": { "kind": "solved", "winner": player_id(ANA), "moves": 2 }
        })
    );
    assert_eq!(
        json_of(
            RoomEvent::GameOver {
                leaders: vec![ANA],
                scores: vec![(ANA, 3)]
            }
            .into()
        ),
        json!({
            "type": "game_over",
            "leaders": [player_id(ANA)],
            "scores": [{ "player": player_id(ANA), "tokens": 3 }]
        })
    );
}

#[test]
fn the_snapshot_carries_the_board_and_robots() {
    let mut room = RoomState::new(fixed_board::board(), fixed_board::starting_robots(), 4);
    room.join(Member {
        id: ANA,
        name: "ana".into(),
        is_guest: true,
    })
    .unwrap();
    let message = ServerMessage::Snapshot {
        you: player_id(ANA),
        room: (&room.view()).into(),
    };
    let value = json_of(message);
    assert_eq!(value["type"], "snapshot");
    assert_eq!(
        value["room"]["members"],
        json!([{ "id": player_id(ANA), "name": "ana", "is_guest": true }])
    );
    assert_eq!(value["room"]["board"]["size"], 16);
    assert_eq!(value["room"]["game"], serde_json::Value::Null);
    let board = &fixed_board::board();
    let targets = value["room"]["board"]["targets"].as_array().unwrap();
    assert_eq!(targets.len(), board.targets().len());
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

#[test]
fn refusals_have_stable_codes() {
    assert_eq!(room_error_code(RoomError::Full), "room_full");
    assert_eq!(
        room_error_code(RoomError::Game(GameError::Round(RoundError::NotYourTurn))),
        "not_your_turn"
    );
    assert_eq!(
        json_of(ServerMessage::error("bad_chat")),
        json!({ "type": "error", "code": "bad_chat" })
    );
    assert_eq!(Robot::from(RobotColor::Green), Robot::Green);
}
