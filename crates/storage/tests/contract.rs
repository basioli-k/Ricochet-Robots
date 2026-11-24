//! One contract, run against every repository implementation.

use domain::{PlayerId, Timestamp};
use storage::{
    Account, FinishedGame, GameResultRepository, InMemoryStore, LeaderboardSort, Locale,
    Participant, PlayerRepository, PlayerStats, Seat, SqliteStore, StorageError,
};

/// Both repositories, as the server will hold them.
trait Store: PlayerRepository + GameResultRepository {}
impl<T: PlayerRepository + GameResultRepository> Store for T {}

fn account(id: u128, username: &str) -> Account {
    Account {
        id: PlayerId::new(id),
        google_subject: format!("google-{id}"),
        username: username.into(),
        locale: Locale::En,
        created_at: Timestamp(1_000),
    }
}

const ANA: PlayerId = PlayerId::new(1);
const BOB: PlayerId = PlayerId::new(2);
const CID: PlayerId = PlayerId::new(3);

async fn with_players(store: &impl Store) {
    for (id, name) in [(1, "ana"), (2, "bob"), (3, "cid")] {
        store.create(&account(id, name)).await.unwrap();
    }
}

fn game(seats: &[(Seat, u32)]) -> FinishedGame {
    FinishedGame {
        finished_at: Timestamp(5_000),
        players: seats
            .iter()
            .map(|(seat, tokens)| Participant {
                seat: seat.clone(),
                tokens: *tokens,
            })
            .collect(),
    }
}

fn stats(games_played: u64, games_won: u64, tokens_won: u64) -> PlayerStats {
    PlayerStats {
        games_played,
        games_won,
        tokens_won,
    }
}

async fn finds_accounts_by_id_and_username_ignoring_case(store: impl Store) {
    store.create(&account(1, "Ana")).await.unwrap();
    assert_eq!(store.by_id(ANA).await.unwrap(), Some(account(1, "Ana")));
    assert_eq!(
        store.by_username("aNA").await.unwrap(),
        Some(account(1, "Ana"))
    );
    assert_eq!(store.by_username("bob").await.unwrap(), None);
    assert_eq!(store.by_id(BOB).await.unwrap(), None);
}

async fn usernames_and_google_accounts_are_unique(store: impl Store) {
    store.create(&account(1, "ana")).await.unwrap();
    // Each case collides on one field only.
    let same_name = Account {
        google_subject: "google-other".into(),
        ..account(2, "ANA")
    };
    assert!(matches!(
        store.create(&same_name).await,
        Err(StorageError::UsernameTaken)
    ));
    let same_google = Account {
        google_subject: "google-1".into(),
        ..account(3, "other")
    };
    assert!(matches!(
        store.create(&same_google).await,
        Err(StorageError::GoogleAccountTaken)
    ));
}

async fn rejects_a_duplicate_account_id(store: impl Store) {
    store.create(&account(1, "ana")).await.unwrap();
    let same_id = Account {
        google_subject: "google-other".into(),
        ..account(1, "other")
    };
    assert!(matches!(
        store.create(&same_id).await,
        Err(StorageError::Database(_))
    ));
}

async fn finds_accounts_by_exact_google_subject(store: impl Store) {
    store.create(&account(1, "ana")).await.unwrap();
    assert_eq!(
        store.by_google_subject("google-1").await.unwrap(),
        Some(account(1, "ana"))
    );
    // Subjects are opaque: no case folding.
    assert_eq!(store.by_google_subject("GOOGLE-1").await.unwrap(), None);
}

async fn changes_locale(store: impl Store) {
    store.create(&account(1, "ana")).await.unwrap();
    store.set_locale(ANA, Locale::Hr).await.unwrap();
    assert_eq!(store.by_id(ANA).await.unwrap().unwrap().locale, Locale::Hr);
    assert!(matches!(
        store.set_locale(BOB, Locale::En).await,
        Err(StorageError::NotFound)
    ));
}

async fn ranked_games_count_tokens_and_wins(store: impl Store) {
    with_players(&store).await;
    store
        .record(&game(&[(Seat::Account(ANA), 3), (Seat::Account(BOB), 1)]))
        .await
        .unwrap();
    store
        .record(&game(&[(Seat::Account(ANA), 0), (Seat::Account(BOB), 2)]))
        .await
        .unwrap();
    assert_eq!(store.stats(ANA).await.unwrap(), stats(2, 1, 3));
    assert_eq!(store.stats(BOB).await.unwrap(), stats(2, 1, 3));
    assert_eq!(store.stats(CID).await.unwrap(), PlayerStats::default());
}

async fn ties_count_as_wins_for_all_top_scorers(store: impl Store) {
    with_players(&store).await;
    let seats = [
        (Seat::Account(ANA), 2),
        (Seat::Account(BOB), 2),
        (Seat::Account(CID), 1),
    ];
    store.record(&game(&seats)).await.unwrap();
    assert_eq!(store.stats(ANA).await.unwrap().games_won, 1);
    assert_eq!(store.stats(BOB).await.unwrap().games_won, 1);
    assert_eq!(store.stats(CID).await.unwrap().games_won, 0);
}

async fn games_with_a_guest_are_stored_but_unranked(store: impl Store) {
    with_players(&store).await;
    let seats = [
        (Seat::Account(ANA), 5),
        (Seat::Guest("visitor".into()), 2),
        (Seat::Account(BOB), 1),
    ];
    let with_guest = game(&seats);
    assert!(!with_guest.is_ranked());
    store.record(&with_guest).await.unwrap();
    // The whole game comes back, guest seat and score included, in order.
    assert_eq!(store.recent_games(10).await.unwrap(), vec![with_guest]);
    assert_eq!(store.stats(ANA).await.unwrap(), PlayerStats::default());
}

async fn recent_games_are_newest_first_and_limited(store: impl Store) {
    with_players(&store).await;
    let at = |t, tokens| FinishedGame {
        finished_at: Timestamp(t),
        players: vec![Participant {
            seat: Seat::Account(ANA),
            tokens,
        }],
    };
    for g in [at(10, 1), at(30, 2), at(20, 3), at(30, 4)] {
        store.record(&g).await.unwrap();
    }
    // Equal times: the later recorded comes first.
    assert_eq!(
        store.recent_games(3).await.unwrap(),
        [at(30, 4), at(30, 2), at(20, 3)]
    );
    assert_eq!(store.recent_games(0).await.unwrap(), []);
}

async fn a_one_player_game_is_unranked(store: impl Store) {
    with_players(&store).await;
    store
        .record(&game(&[(Seat::Account(ANA), 9)]))
        .await
        .unwrap();
    assert_eq!(store.stats(ANA).await.unwrap(), PlayerStats::default());
}

async fn rejects_unknown_accounts_and_duplicate_seats(store: impl Store) {
    with_players(&store).await;
    let unknown = game(&[
        (Seat::Account(ANA), 1),
        (Seat::Account(PlayerId::new(99)), 0),
    ]);
    assert!(matches!(
        store.record(&unknown).await,
        Err(StorageError::NotFound)
    ));
    let twice = game(&[(Seat::Account(ANA), 1), (Seat::Account(ANA), 0)]);
    assert!(matches!(
        store.record(&twice).await,
        Err(StorageError::InvalidGame(_))
    ));
    assert_eq!(store.stats(ANA).await.unwrap(), PlayerStats::default());
}

async fn the_leaderboard_sorts_and_limits(store: impl Store) {
    with_players(&store).await;
    store.create(&account(4, "dan")).await.unwrap(); // no ranked games: not listed
    // Unranked games (here with a guest) never reach the leaderboard.
    let with_guest = [
        (Seat::Account(PlayerId::new(4)), 9),
        (Seat::Guest("visitor".into()), 0),
    ];
    store.record(&game(&with_guest)).await.unwrap();
    store
        .record(&game(&[(Seat::Account(ANA), 1), (Seat::Account(BOB), 8)]))
        .await
        .unwrap();
    store
        .record(&game(&[(Seat::Account(ANA), 3), (Seat::Account(CID), 1)]))
        .await
        .unwrap();
    store
        .record(&game(&[(Seat::Account(ANA), 2), (Seat::Account(CID), 1)]))
        .await
        .unwrap();
    let names = |entries: Vec<storage::LeaderboardEntry>| -> Vec<String> {
        entries.into_iter().map(|e| e.username).collect()
    };
    let by = |sort| store.leaderboard(sort, 10);
    // Ana has the most wins but Bob the most tokens, so each sort differs.
    assert_eq!(
        names(by(LeaderboardSort::GamesWon).await.unwrap()),
        ["ana", "bob", "cid"]
    );
    assert_eq!(
        names(by(LeaderboardSort::GamesPlayed).await.unwrap()),
        ["ana", "cid", "bob"]
    );
    assert_eq!(
        names(by(LeaderboardSort::TokensWon).await.unwrap()),
        ["bob", "ana", "cid"]
    );
    // Equal stats fall back to username order, not creation order.
    store.create(&account(5, "zed")).await.unwrap();
    store.create(&account(6, "amy")).await.unwrap();
    let draw = [
        (Seat::Account(PlayerId::new(5)), 0),
        (Seat::Account(PlayerId::new(6)), 0),
    ];
    store.record(&game(&draw)).await.unwrap();
    assert_eq!(
        names(by(LeaderboardSort::GamesPlayed).await.unwrap()),
        ["ana", "cid", "amy", "bob", "zed"]
    );
    let top = store
        .leaderboard(LeaderboardSort::TokensWon, 1)
        .await
        .unwrap();
    assert_eq!(top.len(), 1);
    assert_eq!(top[0].stats, stats(1, 1, 8));
}

/// Instantiates every contract check for one implementation.
macro_rules! contract {
    ($module:ident, $make:expr) => {
        mod $module {
            use super::*;
            contract!(@tests $make;
                finds_accounts_by_id_and_username_ignoring_case,
                usernames_and_google_accounts_are_unique,
                rejects_a_duplicate_account_id,
                finds_accounts_by_exact_google_subject,
                changes_locale,
                ranked_games_count_tokens_and_wins,
                ties_count_as_wins_for_all_top_scorers,
                games_with_a_guest_are_stored_but_unranked,
                recent_games_are_newest_first_and_limited,
                a_one_player_game_is_unranked,
                rejects_unknown_accounts_and_duplicate_seats,
                the_leaderboard_sorts_and_limits,
            );
        }
    };
    (@tests $make:expr; $($check:ident),* $(,)?) => {
        $(
            #[tokio::test]
            async fn $check() {
                super::$check($make).await;
            }
        )*
    };
}

contract!(in_memory, InMemoryStore::new());
contract!(sqlite, SqliteStore::in_memory().await.unwrap());
