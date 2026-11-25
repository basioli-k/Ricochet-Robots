//! A room's task: commands are answered, events are broadcast in order, and
//! concurrent callers are serialized.

use std::sync::Arc;

use domain::{
    BoardBuilder, Direction, Phase, PlayerId, Position, RobotColor, Robots, Symbol, Target,
    Timestamp,
};
use server::clock::FakeClock;
use server::rooms::actor::{CallError, DeckShuffler, RoomHandle, TICK};
use server::rooms::state::{BIDDING_WINDOW_MS, DEMONSTRATION_MS, Member, RoomError, RoomEvent};
use tokio::sync::broadcast;

const ANA: PlayerId = PlayerId::new(1);
const BOB: PlayerId = PlayerId::new(2);

/// Keeps the deck in board order.
struct InOrder;
impl DeckShuffler for InOrder {
    fn shuffle(&self, _deck: &mut [Target]) {}
}

fn pos(row: u8, col: u8) -> Position {
    Position::new(row, col).unwrap()
}

fn room(capacity: usize, clock: Arc<FakeClock>) -> RoomHandle {
    let board = BoardBuilder::new()
        .target(pos(0, 5), Target::Colored(RobotColor::Red, Symbol::Moon))
        .unwrap()
        .build()
        .unwrap();
    let robots = Robots::new([pos(5, 5), pos(5, 10), pos(10, 0), pos(15, 15)]).unwrap();
    RoomHandle::spawn(board, robots, capacity, clock, Arc::new(InOrder))
}

fn member(id: u128, name: &str) -> Member {
    Member {
        id: PlayerId::new(id),
        name: name.into(),
        is_guest: true,
    }
}

/// The next event, failing after 2 s.
async fn next(events: &mut broadcast::Receiver<RoomEvent>) -> RoomEvent {
    tokio::time::timeout(std::time::Duration::from_secs(2), events.recv())
        .await
        .expect("an event within 2 s")
        .unwrap()
}

#[tokio::test]
async fn a_round_plays_through_the_handle() {
    let clock = Arc::new(FakeClock::at(0));
    let room = room(4, clock.clone());
    let mut events = room.subscribe();
    room.join(member(1, "ana")).await.unwrap();
    room.join(member(2, "bob")).await.unwrap();
    assert!(matches!(next(&mut events).await, RoomEvent::Joined(_)));
    assert!(matches!(next(&mut events).await, RoomEvent::Joined(_)));

    room.start(ANA).await.unwrap();
    assert!(matches!(
        next(&mut events).await,
        RoomEvent::RoundStarted { .. }
    ));
    room.bid(BOB, 1).await.unwrap();
    assert!(matches!(
        next(&mut events).await,
        RoomEvent::BidPlaced {
            player: BOB,
            moves: 1,
            ..
        }
    ));

    clock.advance(BIDDING_WINDOW_MS);
    room.tick().await.unwrap();
    assert_eq!(
        next(&mut events).await,
        RoomEvent::Demonstrating {
            player: BOB,
            bid: 1,
            until: Timestamp(BIDDING_WINDOW_MS + DEMONSTRATION_MS)
        }
    );

    room.move_robot(BOB, RobotColor::Red, Direction::Up)
        .await
        .unwrap();
    assert!(matches!(
        next(&mut events).await,
        RoomEvent::RobotMoved { .. }
    ));
    assert_eq!(
        next(&mut events).await,
        RoomEvent::RoundEnded {
            phase: Phase::Solved {
                winner: BOB,
                moves: 1
            }
        }
    );
}

#[tokio::test]
async fn refusals_come_back_to_the_caller() {
    let room = room(4, Arc::new(FakeClock::at(0)));
    room.join(member(1, "ana")).await.unwrap();
    assert_eq!(
        room.join(member(2, "ANA")).await,
        Err(CallError::Room(RoomError::NameTaken))
    );
    assert_eq!(
        room.bid(ANA, 3).await,
        Err(CallError::Room(RoomError::NotStarted))
    );
}

#[tokio::test]
async fn concurrent_joins_never_exceed_capacity() {
    let room = room(4, Arc::new(FakeClock::at(0)));
    let joins = (0..10u128).map(|i| {
        let room = room.clone();
        tokio::spawn(async move { room.join(member(i + 1, &format!("player{i}"))).await })
    });
    let mut accepted = 0;
    for join in joins {
        match join.await.unwrap() {
            Ok(()) => accepted += 1,
            Err(e) => assert_eq!(e, CallError::Room(RoomError::Full)),
        }
    }
    assert_eq!(accepted, 4);
    assert_eq!(room.members().await.unwrap().len(), 4);
}

/// Waits on the real 250 ms ticker; no `RoomHandle::tick` call.
#[tokio::test]
async fn the_room_advances_its_timers_on_its_own() {
    let clock = Arc::new(FakeClock::at(0));
    let room = room(4, clock.clone());
    room.join(member(1, "ana")).await.unwrap();
    room.start(ANA).await.unwrap();
    room.bid(ANA, 1).await.unwrap();
    let mut events = room.subscribe();
    clock.advance(BIDDING_WINDOW_MS);
    assert_eq!(
        next(&mut events).await,
        RoomEvent::Demonstrating {
            player: ANA,
            bid: 1,
            until: Timestamp(BIDDING_WINDOW_MS + DEMONSTRATION_MS)
        },
        "the next tick (every {TICK:?}) closes bidding"
    );
}

#[tokio::test]
async fn closing_stops_an_empty_room_only() {
    let room = room(4, Arc::new(FakeClock::at(0)));
    room.join(member(1, "ana")).await.unwrap();
    assert_eq!(room.close_if_empty().await, Ok(false));
    assert_eq!(room.members().await.unwrap().len(), 1, "still running");
    room.leave(ANA).await.unwrap();
    let other = room.clone();
    assert_eq!(room.close_if_empty().await, Ok(true));
    assert_eq!(other.join(member(2, "bob")).await, Err(CallError::Closed));
}

#[tokio::test]
async fn the_room_stops_when_every_handle_is_dropped() {
    let room = room(4, Arc::new(FakeClock::at(0)));
    let mut events = room.subscribe();
    drop(room);
    let ended = tokio::time::timeout(std::time::Duration::from_secs(2), events.recv())
        .await
        .expect("the task ends");
    assert_eq!(ended, Err(broadcast::error::RecvError::Closed));
}

#[tokio::test]
async fn chat_and_the_view_go_through_the_handle() {
    let room = room(4, Arc::new(FakeClock::at(0)));
    let mut events = room.subscribe();
    room.join(member(1, "ana")).await.unwrap();
    next(&mut events).await;
    room.chat(ANA, " hello ".into()).await.unwrap();
    assert_eq!(
        next(&mut events).await,
        RoomEvent::Chat {
            player: ANA,
            text: "hello".into()
        }
    );
    assert_eq!(
        room.chat(BOB, "hi".into()).await,
        Err(CallError::Room(RoomError::NotMember))
    );
    let view = room.view().await.unwrap();
    assert_eq!(view.members, [member(1, "ana")]);
    assert!(view.game.is_none());
}
