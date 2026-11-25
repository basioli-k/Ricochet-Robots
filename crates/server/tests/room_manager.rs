//! Creating rooms within limits, finding them, and closing idle ones.

use std::sync::Arc;

use domain::{PlayerId, fixed_board};
use server::clock::FakeClock;
use server::rooms::actor::RandomShuffler;
use server::rooms::manager::{CreateError, EMPTY_CLOSE_MS, RoomLimits, RoomManager};
use server::rooms::state::Member;

fn manager(global: usize, per_owner: usize, clock: Arc<FakeClock>) -> RoomManager {
    RoomManager::new(
        RoomLimits {
            global,
            per_owner,
            capacity: 4,
        },
        || (fixed_board::board(), fixed_board::starting_robots()),
        clock,
        Arc::new(RandomShuffler),
    )
}

#[tokio::test]
async fn owners_and_the_server_have_room_caps() {
    let rooms = manager(3, 2, Arc::new(FakeClock::at(0)));
    rooms.create(&["a"]).unwrap();
    rooms.create(&["a"]).unwrap();
    assert_eq!(rooms.create(&["a"]).err(), Some(CreateError::OwnerLimit));
    rooms.create(&["b"]).unwrap();
    assert_eq!(rooms.create(&["c"]).err(), Some(CreateError::TooManyRooms));
}

#[tokio::test]
async fn a_room_counts_against_each_of_its_owners() {
    let rooms = manager(10, 1, Arc::new(FakeClock::at(0)));
    rooms.create(&["player-1", "203.0.113.7"]).unwrap();
    assert_eq!(
        rooms.create(&["player-2", "203.0.113.7"]).err(),
        Some(CreateError::OwnerLimit),
        "same address, new session"
    );
    assert_eq!(
        rooms.create(&["player-1", "198.51.100.1"]).err(),
        Some(CreateError::OwnerLimit),
        "same session, new address"
    );
    rooms.create(&["player-2", "198.51.100.1"]).unwrap();
}

#[tokio::test]
async fn rooms_are_found_and_listed_with_member_counts() {
    let rooms = manager(10, 2, Arc::new(FakeClock::at(0)));
    let (id, handle) = rooms.create(&["a"]).unwrap();
    handle
        .join(Member {
            id: PlayerId::new(1),
            name: "ana".into(),
            is_guest: true,
        })
        .await
        .unwrap();
    assert!(rooms.get(&id).is_some());
    assert!(rooms.get("missing").is_none());
    let list = rooms.list().await;
    assert_eq!(list.len(), 1);
    assert_eq!((list[0].id.as_str(), list[0].members), (id.as_str(), 1));
}

#[tokio::test]
async fn empty_rooms_close_after_the_idle_period() {
    let clock = Arc::new(FakeClock::at(0));
    let rooms = manager(10, 5, clock.clone());
    let (empty, _) = rooms.create(&["a"]).unwrap();
    let (busy, handle) = rooms.create(&["a"]).unwrap();
    handle
        .join(Member {
            id: PlayerId::new(1),
            name: "ana".into(),
            is_guest: true,
        })
        .await
        .unwrap();

    assert_eq!(
        rooms.close_idle().await,
        0,
        "the countdown starts at the first check"
    );
    clock.advance(EMPTY_CLOSE_MS - 1);
    assert_eq!(rooms.close_idle().await, 0);
    clock.advance(1);
    assert_eq!(rooms.close_idle().await, 1);
    assert!(rooms.get(&empty).is_none());
    assert!(rooms.get(&busy).is_some(), "a room with members stays open");
}

#[tokio::test]
async fn a_member_joining_resets_the_countdown() {
    let clock = Arc::new(FakeClock::at(0));
    let rooms = manager(10, 5, clock.clone());
    let (id, handle) = rooms.create(&["a"]).unwrap();
    rooms.close_idle().await;
    clock.advance(EMPTY_CLOSE_MS - 1);
    handle
        .join(Member {
            id: PlayerId::new(1),
            name: "ana".into(),
            is_guest: true,
        })
        .await
        .unwrap();
    rooms.close_idle().await;
    handle.leave(PlayerId::new(1)).await.unwrap();
    clock.advance(1);
    assert_eq!(rooms.close_idle().await, 0, "emptied again just now");
    assert!(rooms.get(&id).is_some());
}
