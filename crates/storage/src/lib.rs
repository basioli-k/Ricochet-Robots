//! Persistence behind repository traits, with an SQLite implementation and
//! an in-memory one for tests.

mod memory;
mod model;
mod repository;
mod sqlite;

pub use memory::InMemoryStore;
pub use model::{
    Account, FinishedGame, LeaderboardEntry, LeaderboardSort, Locale, Participant, PlayerStats,
    Seat,
};
pub use repository::{GameResultRepository, PlayerRepository, StorageError};
pub use sqlite::SqliteStore;
