//! The SQLite store on a real file: data survives reconnecting and
//! migrations are safe to run again.

use domain::{PlayerId, Timestamp};
use storage::{Account, Locale, PlayerRepository, SqliteStore};

#[tokio::test]
async fn data_survives_reconnecting() {
    let path = std::env::temp_dir().join(format!("ricochet-test-{}.db", std::process::id()));
    let url = format!("sqlite://{}", path.display());
    let ana = Account {
        id: PlayerId::new(7),
        google_subject: "google-7".into(),
        username: "ana".into(),
        locale: Locale::Hr,
        created_at: Timestamp(1),
    };
    {
        let store = SqliteStore::connect(&url).await.unwrap();
        store.create(&ana).await.unwrap();
    }
    // Reconnecting re-runs the migrator, which must skip applied migrations.
    let store = SqliteStore::connect(&url).await.unwrap();
    let found = store.by_id(PlayerId::new(7)).await.unwrap();
    drop(store);
    for suffix in ["", "-wal", "-shm"] {
        let _ = std::fs::remove_file(format!("{}{suffix}", path.display()));
    }
    assert_eq!(found, Some(ana));
}
