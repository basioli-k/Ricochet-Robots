//! The room state machine, driven with explicit timestamps.

use domain::{
    Board, BoardBuilder, Direction, GameError, Phase, PlayerId, Position, RobotColor, Robots,
    RoundError, Shuffle, Symbol, Target, Timestamp,
};
use server::rooms::state::{
    BIDDING_WINDOW_MS, DEMONSTRATION_MS, IDLE_ROUND_MS, Member, RESULT_PAUSE_MS, RoomError,
    RoomEvent, RoomState,
};

const ANA: PlayerId = PlayerId::new(1);
const BOB: PlayerId = PlayerId::new(2);
const RED_MOON: Target = Target::Colored(RobotColor::Red, Symbol::Moon);

struct InOrder;
impl Shuffle for InOrder {
    fn shuffle(&mut self, _deck: &mut [Target]) {}
}

fn pos(row: u8, col: u8) -> Position {
    Position::new(row, col).unwrap()
}

/// Red Moon at (0,5) then the vortex at (0,10); Red at (5,5) solves the
/// moon with Up, Blue at (5,10) the vortex with Up.
fn board() -> Board {
    BoardBuilder::new()
        .target(pos(0, 5), RED_MOON)
        .and_then(|b| b.target(pos(0, 10), Target::Vortex))
        .unwrap()
        .build()
        .unwrap()
}

fn robots() -> Robots {
    Robots::new([pos(5, 5), pos(5, 10), pos(10, 0), pos(15, 15)]).unwrap()
}

fn member(id: PlayerId, name: &str) -> Member {
    Member {
        id,
        name: name.into(),
        is_guest: true,
    }
}

fn t(ms: u64) -> Timestamp {
    Timestamp(ms)
}

/// Ana and Bob in a started room, bidding on the red moon from t=0.
fn started() -> RoomState {
    let mut room = RoomState::new(board(), robots(), 4);
    room.join(member(ANA, "ana")).unwrap();
    room.join(member(BOB, "bob")).unwrap();
    room.start(ANA, &mut InOrder, t(0)).unwrap();
    room
}

fn phase(room: &RoomState) -> Phase {
    room.game().unwrap().round().phase()
}

#[test]
fn names_are_unique_ignoring_case_and_capacity_is_enforced() {
    let mut room = RoomState::new(board(), robots(), 2);
    assert_eq!(
        room.join(member(ANA, "ana")).unwrap(),
        [RoomEvent::Joined(member(ANA, "ana"))]
    );
    assert_eq!(room.join(member(BOB, "ANA")), Err(RoomError::NameTaken));
    assert_eq!(
        room.join(member(ANA, "ana")).unwrap(),
        [],
        "rejoining is a no-op"
    );
    room.join(member(BOB, "bob")).unwrap();
    assert_eq!(
        room.join(member(PlayerId::new(3), "cid")),
        Err(RoomError::Full)
    );
}

#[test]
fn only_members_act_and_only_once_started() {
    let mut room = RoomState::new(board(), robots(), 4);
    room.join(member(ANA, "ana")).unwrap();
    assert_eq!(room.bid(ANA, 3, t(0)), Err(RoomError::NotStarted));
    assert_eq!(
        room.start(BOB, &mut InOrder, t(0)),
        Err(RoomError::NotMember)
    );
    room.start(ANA, &mut InOrder, t(0)).unwrap();
    assert_eq!(
        room.start(ANA, &mut InOrder, t(0)),
        Err(RoomError::AlreadyStarted)
    );
    assert_eq!(room.bid(BOB, 3, t(0)), Err(RoomError::NotMember));
}

#[test]
fn a_full_round_runs_on_ticks() {
    let mut room = started();
    let events = room.bid(ANA, 1, t(1_000)).unwrap();
    assert_eq!(
        events,
        [RoomEvent::BidPlaced {
            player: ANA,
            moves: 1,
            closes_at: t(1_000 + BIDDING_WINDOW_MS)
        }]
    );
    assert_eq!(
        room.tick(t(1_000 + BIDDING_WINDOW_MS - 1)),
        [],
        "the window is still open"
    );
    let close = 1_000 + BIDDING_WINDOW_MS;
    assert_eq!(
        room.tick(t(close)),
        [RoomEvent::Demonstrating {
            player: ANA,
            bid: 1
        }]
    );

    let events = room
        .move_robot(ANA, RobotColor::Red, Direction::Up, t(close + 10))
        .unwrap();
    assert!(matches!(
        events[0],
        RoomEvent::RobotMoved {
            robot: RobotColor::Red,
            ..
        }
    ));
    assert_eq!(
        events[1],
        RoomEvent::RoundEnded {
            phase: Phase::Solved {
                winner: ANA,
                moves: 1
            }
        }
    );

    assert_eq!(
        room.tick(t(close + 10 + RESULT_PAUSE_MS - 1)),
        [],
        "the result pause"
    );
    let next = room.tick(t(close + 10 + RESULT_PAUSE_MS));
    assert!(matches!(
        next[..],
        [RoomEvent::RoundStarted {
            target: Target::Vortex,
            ..
        }]
    ));
}

#[test]
fn a_slow_demonstrator_concedes_and_the_round_can_end_unsolved() {
    let mut room = started();
    room.bid(ANA, 2, t(0)).unwrap();
    room.tick(t(BIDDING_WINDOW_MS));
    let late = BIDDING_WINDOW_MS + DEMONSTRATION_MS;
    assert_eq!(room.tick(t(late - 1)), []);
    assert_eq!(
        room.tick(t(late)),
        [RoomEvent::RoundEnded {
            phase: Phase::Unsolved
        }]
    );
}

#[test]
fn moving_does_not_restart_the_demonstration_clock() {
    let mut room = started();
    room.bid(ANA, 2, t(0)).unwrap();
    let close = BIDDING_WINDOW_MS;
    room.tick(t(close));
    let events = room
        .move_robot(ANA, RobotColor::Red, Direction::Down, t(close + 50_000))
        .unwrap();
    assert_eq!(events.len(), 1, "only the move: {events:?}");
    assert_eq!(
        room.tick(t(close + DEMONSTRATION_MS)),
        [RoomEvent::RoundEnded {
            phase: Phase::Unsolved
        }]
    );
}

#[test]
fn an_unbid_round_is_abandoned_after_the_idle_timeout() {
    let mut room = started();
    assert_eq!(room.tick(t(IDLE_ROUND_MS - 1)), []);
    assert_eq!(
        room.tick(t(IDLE_ROUND_MS)),
        [RoomEvent::RoundEnded {
            phase: Phase::Unsolved
        }]
    );
}

#[test]
fn a_leaving_demonstrator_concedes() {
    let mut room = started();
    room.bid(BOB, 1, t(0)).unwrap();
    room.tick(t(BIDDING_WINDOW_MS));
    assert_eq!(
        phase(&room),
        Phase::Demonstrating {
            player: BOB,
            bid: 1,
            moves_made: 0
        }
    );
    let events = room.leave(BOB, t(BIDDING_WINDOW_MS + 1)).unwrap();
    assert_eq!(
        events,
        [
            RoomEvent::Left(BOB),
            RoomEvent::RoundEnded {
                phase: Phase::Unsolved
            }
        ]
    );
}

#[test]
fn a_bidder_who_left_is_skipped_when_bidding_closes() {
    let mut room = started();
    room.bid(BOB, 1, t(0)).unwrap();
    room.bid(ANA, 2, t(10)).unwrap();
    room.leave(BOB, t(20)).unwrap();
    assert_eq!(
        room.tick(t(BIDDING_WINDOW_MS)),
        [RoomEvent::Demonstrating {
            player: ANA,
            bid: 2
        }]
    );
}

#[test]
fn the_last_round_ends_the_game_with_its_leaders() {
    let mut room = started();
    let mut now = 0;
    for robot in [RobotColor::Red, RobotColor::Blue] {
        room.bid(ANA, 1, t(now)).unwrap();
        now += BIDDING_WINDOW_MS;
        room.tick(t(now));
        room.move_robot(ANA, robot, Direction::Up, t(now)).unwrap();
        now += RESULT_PAUSE_MS;
        room.tick(t(now));
    }
    assert!(room.game().unwrap().is_over());
    assert_eq!(room.tick(t(now + 1)), [], "nothing happens after the game");
}

#[test]
fn game_refusals_pass_through_unchanged() {
    let mut room = started();
    assert_eq!(
        room.move_robot(ANA, RobotColor::Red, Direction::Up, t(0)),
        Err(RoomError::Game(GameError::Round(RoundError::WrongPhase)))
    );
}
