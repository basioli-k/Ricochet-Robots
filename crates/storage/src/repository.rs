//! The storage interfaces the server depends on.

use async_trait::async_trait;
use domain::{PlayerId, Timestamp};

use crate::model::{Account, FinishedGame, LeaderboardEntry, LeaderboardSort, Locale, PlayerStats};

/// Why a storage call failed.
#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    /// Usernames are unique ignoring case.
    #[error("username is taken")]
    UsernameTaken,
    /// Email addresses are unique ignoring case.
    #[error("email is already registered")]
    EmailTaken,
    #[error("no such account")]
    NotFound,
    /// The game record breaks a rule, e.g. the same account in two seats.
    #[error("invalid game record: {0}")]
    InvalidGame(&'static str),
    #[error("database error")]
    Database(#[source] Box<dyn std::error::Error + Send + Sync>),
}

/// Registered accounts.
#[async_trait]
pub trait PlayerRepository: Send + Sync {
    /// # Errors
    /// [`StorageError::UsernameTaken`] or [`StorageError::EmailTaken`] when
    /// either matches an existing account ignoring case.
    async fn create(&self, account: &Account) -> Result<(), StorageError>;

    /// # Errors
    /// Only on database failure.
    async fn by_id(&self, id: PlayerId) -> Result<Option<Account>, StorageError>;

    /// Looks the username up ignoring case.
    ///
    /// # Errors
    /// Only on database failure.
    async fn by_username(&self, username: &str) -> Result<Option<Account>, StorageError>;

    /// # Errors
    /// [`StorageError::NotFound`] for an unknown account.
    async fn confirm(&self, id: PlayerId, at: Timestamp) -> Result<(), StorageError>;

    /// # Errors
    /// [`StorageError::NotFound`] for an unknown account.
    async fn set_locale(&self, id: PlayerId, locale: Locale) -> Result<(), StorageError>;
}

/// Finished games and the stats derived from them.
#[async_trait]
pub trait GameResultRepository: Send + Sync {
    /// Stores a game once it has ended; its ranked flag and winners are
    /// fixed at this point (see [`FinishedGame`]).
    ///
    /// # Errors
    /// [`StorageError::NotFound`] if a seat names an unknown account,
    /// [`StorageError::InvalidGame`] if an account holds two seats.
    async fn record(&self, game: &FinishedGame) -> Result<(), StorageError>;

    /// Totals over ranked games only; zero for accounts that played none.
    ///
    /// # Errors
    /// Only on database failure.
    async fn stats(&self, id: PlayerId) -> Result<PlayerStats, StorageError>;

    /// Accounts with at least one ranked game, best first by `sort`, then
    /// by username; at most `limit` entries.
    ///
    /// # Errors
    /// Only on database failure.
    async fn leaderboard(
        &self,
        sort: LeaderboardSort,
        limit: u32,
    ) -> Result<Vec<LeaderboardEntry>, StorageError>;
}

/// Rejects a game where an account holds more than one seat.
pub(crate) fn check_seats(game: &FinishedGame) -> Result<(), StorageError> {
    let mut ids: Vec<u128> = game
        .players
        .iter()
        .filter_map(|p| match p.seat {
            crate::Seat::Account(id) => Some(id.raw()),
            crate::Seat::Guest(_) => None,
        })
        .collect();
    let total = ids.len();
    ids.sort_unstable();
    ids.dedup();
    if ids.len() == total {
        Ok(())
    } else {
        Err(StorageError::InvalidGame("an account holds two seats"))
    }
}
