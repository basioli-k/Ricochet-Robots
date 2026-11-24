//! Persistence behind repository traits, with an SQLite implementation and
//! an in-memory one for tests.

mod model;

pub use model::{
    Account, FinishedGame, LeaderboardEntry, LeaderboardSort, Locale, Participant, PlayerStats,
    Seat,
};
