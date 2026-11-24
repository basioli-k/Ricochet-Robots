//! The SQLite store on a real file: data survives reconnecting and
//! migrations are safe to run again.

use std::path::PathBuf;

use domain::{PlayerId, Timestamp};
use storage::{Account, Locale, PlayerRepository, SqliteStore};

/// A fresh directory for one test, removed on drop even if the test panics.
struct TempDir(PathBuf);

impl TempDir {
    fn new(name: &str) -> Self {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.as_nanos());
        let dir = std::env::temp_dir().join(format!("{name}-{}-{nanos}", std::process::id()));
        // create_dir (not _all) fails if it exists, so a stale run can't leak in.
        std::fs::create_dir(&dir).unwrap();
        Self(dir)
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

#[tokio::test]
async fn data_survives_reconnecting() {
    let dir = TempDir::new("ricochet-sqlite-test");
    let url = format!("sqlite://{}", dir.0.join("store.db").display());
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
    assert_eq!(store.by_id(PlayerId::new(7)).await.unwrap(), Some(ana));
}
