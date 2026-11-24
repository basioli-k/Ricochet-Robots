//! An in-memory store for tests; same contract as the SQLite one.

use std::sync::Mutex;

use async_trait::async_trait;
use domain::{PlayerId, Timestamp};

use crate::model::{
    Account, FinishedGame, LeaderboardEntry, LeaderboardSort, Locale, PlayerStats, Seat,
};
use crate::repository::{GameResultRepository, PlayerRepository, StorageError, check_seats};

/// Accounts and games in a `Mutex`; implements both repositories.
#[derive(Debug, Default)]
pub struct InMemoryStore {
    inner: Mutex<Inner>,
}

#[derive(Debug, Default)]
struct Inner {
    accounts: Vec<Account>,
    /// Per stored ranked game: (account, tokens, won).
    ranked_seats: Vec<(PlayerId, u32, bool)>,
}

impl InMemoryStore {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, Inner> {
        // A panic while holding the lock can't leave `Inner` half-updated:
        // every mutation is a single push or field assignment.
        self.inner
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}

impl Inner {
    fn account_mut(&mut self, id: PlayerId) -> Result<&mut Account, StorageError> {
        self.accounts
            .iter_mut()
            .find(|a| a.id == id)
            .ok_or(StorageError::NotFound)
    }

    fn stats(&self, id: PlayerId) -> PlayerStats {
        self.ranked_seats
            .iter()
            .filter(|(who, _, _)| *who == id)
            .fold(PlayerStats::default(), |s, &(_, tokens, won)| PlayerStats {
                games_played: s.games_played + 1,
                games_won: s.games_won + u64::from(won),
                tokens_won: s.tokens_won + u64::from(tokens),
            })
    }
}

#[async_trait]
impl PlayerRepository for InMemoryStore {
    async fn create(&self, account: &Account) -> Result<(), StorageError> {
        let mut inner = self.lock();
        if inner
            .accounts
            .iter()
            .any(|a| a.username.eq_ignore_ascii_case(&account.username))
        {
            return Err(StorageError::UsernameTaken);
        }
        if inner
            .accounts
            .iter()
            .any(|a| a.email.eq_ignore_ascii_case(&account.email))
        {
            return Err(StorageError::EmailTaken);
        }
        inner.accounts.push(account.clone());
        Ok(())
    }

    async fn by_id(&self, id: PlayerId) -> Result<Option<Account>, StorageError> {
        Ok(self.lock().accounts.iter().find(|a| a.id == id).cloned())
    }

    async fn by_username(&self, username: &str) -> Result<Option<Account>, StorageError> {
        Ok(self
            .lock()
            .accounts
            .iter()
            .find(|a| a.username.eq_ignore_ascii_case(username))
            .cloned())
    }

    async fn confirm(&self, id: PlayerId, at: Timestamp) -> Result<(), StorageError> {
        self.lock().account_mut(id)?.confirmed_at = Some(at);
        Ok(())
    }

    async fn set_locale(&self, id: PlayerId, locale: Locale) -> Result<(), StorageError> {
        self.lock().account_mut(id)?.locale = locale;
        Ok(())
    }
}

#[async_trait]
impl GameResultRepository for InMemoryStore {
    async fn record(&self, game: &FinishedGame) -> Result<(), StorageError> {
        check_seats(game)?;
        let mut inner = self.lock();
        for p in &game.players {
            if let Seat::Account(id) = p.seat {
                inner.account_mut(id)?;
            }
        }
        if game.is_ranked() {
            for (p, won) in game.players.iter().zip(game.winners()) {
                if let Seat::Account(id) = p.seat {
                    inner.ranked_seats.push((id, p.tokens, won));
                }
            }
        }
        Ok(())
    }

    async fn stats(&self, id: PlayerId) -> Result<PlayerStats, StorageError> {
        Ok(self.lock().stats(id))
    }

    async fn leaderboard(
        &self,
        sort: LeaderboardSort,
        limit: u32,
    ) -> Result<Vec<LeaderboardEntry>, StorageError> {
        let inner = self.lock();
        let mut entries: Vec<LeaderboardEntry> = inner
            .accounts
            .iter()
            .map(|a| LeaderboardEntry {
                username: a.username.clone(),
                stats: inner.stats(a.id),
            })
            .filter(|e| e.stats.games_played > 0)
            .collect();
        entries.sort_by(|a, b| {
            sort_key(sort, b.stats)
                .cmp(&sort_key(sort, a.stats))
                .then_with(|| a.username.cmp(&b.username))
        });
        entries.truncate(usize::try_from(limit).unwrap_or(usize::MAX));
        Ok(entries)
    }
}

fn sort_key(sort: LeaderboardSort, stats: PlayerStats) -> u64 {
    match sort {
        LeaderboardSort::GamesWon => stats.games_won,
        LeaderboardSort::GamesPlayed => stats.games_played,
        LeaderboardSort::TokensWon => stats.tokens_won,
    }
}
