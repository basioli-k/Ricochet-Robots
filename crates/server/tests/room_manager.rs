//! Creating rooms within limits, finding them, and closing idle ones.

use std::sync::Arc;

use domain::{PlayerId, fixed_board};
use server::clock::FakeClock;
use server::rate_limit::SlidingWindow;
use server::rooms::actor::{CallError, RandomShuffler};
use server::rooms::manager::{CreateError, Creator, EMPTY_CLOSE_MS, RoomLimits, RoomManager};
use server::rooms::state::Member;
use storage::InMemoryStore;

fn manager(global: usize, per_owner: usize, clock: Arc<FakeClock>) -> RoomManager {
    limited(global, per_owner, usize::MAX, clock)
}

/// A manager allowing `creates` room creations per client per minute.
fn limited(global: usize, per_owner: usize, creates: usize, clock: Arc<FakeClock>) -> RoomManager {
    sharing(
        global,
        per_owner,
        Arc::new(SlidingWindow::new(creates, 60_000)),
        clock,
    )
}

fn sharing(
    global: usize,
    per_owner: usize,
    create_limiter: Arc<SlidingWindow>,
    clock: Arc<FakeClock>,
) -> RoomManager {
    RoomManager::new(
        RoomLimits {
            global,
            per_owner,
            capacity: 4,
        },
        || (fixed_board::board(), fixed_board::starting_robots()),
        clock,
        Arc::new(RandomShuffler),
        create_limiter,
        Arc::new(InMemoryStore::new()),
    )
}

/// A creator whose player and client keys are both derived from `name`.
fn by(name: &str) -> Creator<'_> {
    Creator {
        player: name,
        client: name,
    }
}

#[tokio::test]
async fn owners_and_the_server_have_room_caps() {
    let rooms = manager(3, 2, Arc::new(FakeClock::at(0)));
    rooms.create(by("a")).unwrap();
    rooms.create(by("a")).unwrap();
    assert_eq!(rooms.create(by("a")).err(), Some(CreateError::OwnerLimit));
    rooms.create(by("b")).unwrap();
    assert_eq!(rooms.create(by("c")).err(), Some(CreateError::TooManyRooms));
}

#[tokio::test]
async fn a_room_counts_against_its_player_and_client() {
    let rooms = manager(10, 1, Arc::new(FakeClock::at(0)));
    rooms
        .create(Creator {
            player: "player-1",
            client: "203.0.113.7",
        })
        .unwrap();
    assert_eq!(
        rooms
            .create(Creator {
                player: "player-2",
                client: "203.0.113.7"
            })
            .err(),
        Some(CreateError::OwnerLimit),
        "same address, new session"
    );
    assert_eq!(
        rooms
            .create(Creator {
                player: "player-1",
                client: "198.51.100.1"
            })
            .err(),
        Some(CreateError::OwnerLimit),
        "same session, new address"
    );
    rooms
        .create(Creator {
            player: "player-2",
            client: "198.51.100.1",
        })
        .unwrap();
    rooms
        .create(Creator {
            player: "203.0.113.9",
            client: "x",
        })
        .unwrap();
    rooms
        .create(Creator {
            player: "y",
            client: "203.0.113.9",
        })
        .expect("a player ID never counts as a client key");
}

#[tokio::test]
async fn room_creation_is_rate_limited_per_client() {
    let clock = Arc::new(FakeClock::at(0));
    let limiter = Arc::new(SlidingWindow::new(2, 60_000));
    let full = sharing(1, 5, limiter.clone(), clock.clone());
    let open = sharing(10, 5, limiter, clock.clone());
    let from = |player| Creator {
        player,
        client: "203.0.113.7",
    };
    full.create(from("p1")).unwrap();
    // Refused by a cap, so it doesn't use up the allowance.
    assert_eq!(
        full.create(from("p2")).err(),
        Some(CreateError::TooManyRooms)
    );
    open.create(from("p3")).unwrap();
    // Nor is one refused by the owner cap, and the cap's error wins.
    let limiter = Arc::new(SlidingWindow::new(2, 60_000));
    let one_each = sharing(10, 1, limiter.clone(), clock.clone());
    let roomy = sharing(10, 5, limiter, clock.clone());
    one_each.create(from("p3")).unwrap();
    assert_eq!(
        one_each.create(from("p3")).err(),
        Some(CreateError::OwnerLimit)
    );
    roomy
        .create(from("p5"))
        .expect("the refusal left the second creation free");
    assert_eq!(
        one_each.create(from("p3")).err(),
        Some(CreateError::OwnerLimit),
        "the cap's error wins over the limiter's"
    );
    assert_eq!(
        open.create(from("p4")).err(),
        Some(CreateError::RateLimited)
    );
    open.create(Creator {
        player: "p4",
        client: "198.51.100.1",
    })
    .expect("other clients are unaffected");
    clock.advance(60_000);
    open.create(from("p4")).unwrap();
}

#[tokio::test]
async fn rooms_are_found_and_listed_with_member_counts() {
    let rooms = manager(10, 2, Arc::new(FakeClock::at(0)));
    let (id, handle) = rooms.create(by("a")).unwrap();
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
    let (empty, kept) = rooms.create(by("a")).unwrap();
    let (busy, handle) = rooms.create(by("a")).unwrap();
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
    assert_eq!(
        kept.members().await,
        Err(CallError::Closed),
        "its task stopped, so a handle held elsewhere is closed too"
    );
    assert!(rooms.get(&busy).is_some(), "a room with members stays open");
}

#[tokio::test]
async fn a_member_joining_resets_the_countdown() {
    let clock = Arc::new(FakeClock::at(0));
    let rooms = manager(10, 5, clock.clone());
    let (id, handle) = rooms.create(by("a")).unwrap();
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
