//! The SQLite implementation of both repositories.

use std::str::FromStr;

use async_trait::async_trait;
use domain::{PlayerId, Timestamp};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions, SqliteRow};
use sqlx::{Row, Sqlite, Transaction};

use crate::model::{
    Account, FinishedGame, LeaderboardEntry, LeaderboardSort, Locale, PlayerStats, Seat,
};
use crate::repository::{GameResultRepository, PlayerRepository, StorageError, check_seats};

/// Accounts and games in SQLite. Migrations run on connect.
#[derive(Debug, Clone)]
pub struct SqliteStore {
    pool: SqlitePool,
}

impl SqliteStore {
    /// Opens (creating if missing) the database at `url`, e.g.
    /// `sqlite://ricochet.db`, and applies pending migrations.
    ///
    /// # Errors
    /// If the database can't be opened or migrated.
    pub async fn connect(url: &str) -> Result<Self, StorageError> {
        let options = SqliteConnectOptions::from_str(url)
            .map_err(db)?
            .create_if_missing(true)
            .foreign_keys(true);
        Self::open(SqlitePoolOptions::new(), options).await
    }

    /// A private in-memory database, for tests.
    ///
    /// # Errors
    /// If migrations fail.
    pub async fn in_memory() -> Result<Self, StorageError> {
        let options = SqliteConnectOptions::from_str("sqlite::memory:")
            .map_err(db)?
            .foreign_keys(true);
        // Each connection to `:memory:` is its own database, so keep one.
        Self::open(SqlitePoolOptions::new().max_connections(1), options).await
    }

    async fn open(
        pool: SqlitePoolOptions,
        options: SqliteConnectOptions,
    ) -> Result<Self, StorageError> {
        let pool = pool.connect_with(options).await.map_err(db)?;
        sqlx::migrate!("../../migrations")
            .run(&pool)
            .await
            .map_err(db)?;
        Ok(Self { pool })
    }
}

fn db(e: impl std::error::Error + Send + Sync + 'static) -> StorageError {
    StorageError::Database(Box::new(e))
}

fn id_text(id: PlayerId) -> String {
    format!("{:032x}", id.raw())
}

fn millis(t: Timestamp) -> Result<i64, StorageError> {
    i64::try_from(t.0).map_err(db)
}

/// Maps a unique-constraint failure on `accounts` to the matching error.
fn create_error(e: sqlx::Error) -> StorageError {
    if let sqlx::Error::Database(ref inner) = e {
        let message = inner.message();
        if message.contains("accounts.username_key") {
            return StorageError::UsernameTaken;
        }
        if message.contains("accounts.email_key") {
            return StorageError::EmailTaken;
        }
    }
    db(e)
}

fn account_from(row: &SqliteRow) -> Result<Account, StorageError> {
    let id: String = row.try_get("id").map_err(db)?;
    let locale: String = row.try_get("locale").map_err(db)?;
    let timestamp = |column: &str| -> Result<Option<Timestamp>, StorageError> {
        let value: Option<i64> = row.try_get(column).map_err(db)?;
        value
            .map(|v| u64::try_from(v).map(Timestamp).map_err(db))
            .transpose()
    };
    Ok(Account {
        id: PlayerId::new(u128::from_str_radix(&id, 16).map_err(db)?),
        username: row.try_get("username").map_err(db)?,
        email: row.try_get("email").map_err(db)?,
        password_hash: row.try_get("password_hash").map_err(db)?,
        locale: Locale::from_code(&locale).ok_or(StorageError::InvalidGame("unknown locale"))?,
        created_at: timestamp("created_at")?.ok_or(StorageError::NotFound)?,
        confirmed_at: timestamp("confirmed_at")?,
    })
}

fn stats_from(row: &SqliteRow) -> Result<PlayerStats, StorageError> {
    let count = |column: &str| -> Result<u64, StorageError> {
        let value: i64 = row.try_get(column).map_err(db)?;
        u64::try_from(value).map_err(db)
    };
    Ok(PlayerStats {
        games_played: count("played")?,
        games_won: count("won")?,
        tokens_won: count("tokens")?,
    })
}

/// The leaderboard query ordered by one of its own result columns.
macro_rules! leaderboard_sql {
    ($column:literal) => {
        concat!(
            "SELECT a.username, COUNT(*) AS played, SUM(p.won) AS won, SUM(p.tokens) AS tokens ",
            "FROM accounts a JOIN game_players p ON p.account_id = a.id ",
            "JOIN games g ON g.id = p.game_id AND g.ranked = 1 ",
            "GROUP BY a.id ORDER BY ",
            $column,
            " DESC, a.username ASC LIMIT ?"
        )
    };
}

const ACCOUNT_COLUMNS: &str =
    "id, username, email, password_hash, locale, created_at, confirmed_at";

#[async_trait]
impl PlayerRepository for SqliteStore {
    async fn create(&self, account: &Account) -> Result<(), StorageError> {
        sqlx::query(
            "INSERT INTO accounts (id, username, username_key, email, email_key, password_hash, \
             locale, created_at, confirmed_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(id_text(account.id))
        .bind(&account.username)
        .bind(account.username.to_ascii_lowercase())
        .bind(&account.email)
        .bind(account.email.to_ascii_lowercase())
        .bind(&account.password_hash)
        .bind(account.locale.code())
        .bind(millis(account.created_at)?)
        .bind(account.confirmed_at.map(millis).transpose()?)
        .execute(&self.pool)
        .await
        .map_err(create_error)?;
        Ok(())
    }

    async fn by_id(&self, id: PlayerId) -> Result<Option<Account>, StorageError> {
        let sql = format!("SELECT {ACCOUNT_COLUMNS} FROM accounts WHERE id = ?");
        let row = sqlx::query(sqlx::AssertSqlSafe(sql))
            .bind(id_text(id))
            .fetch_optional(&self.pool)
            .await
            .map_err(db)?;
        row.as_ref().map(account_from).transpose()
    }

    async fn by_username(&self, username: &str) -> Result<Option<Account>, StorageError> {
        let sql = format!("SELECT {ACCOUNT_COLUMNS} FROM accounts WHERE username_key = ?");
        let row = sqlx::query(sqlx::AssertSqlSafe(sql))
            .bind(username.to_ascii_lowercase())
            .fetch_optional(&self.pool)
            .await
            .map_err(db)?;
        row.as_ref().map(account_from).transpose()
    }

    async fn confirm(&self, id: PlayerId, at: Timestamp) -> Result<(), StorageError> {
        let done = sqlx::query("UPDATE accounts SET confirmed_at = ? WHERE id = ?")
            .bind(millis(at)?)
            .bind(id_text(id))
            .execute(&self.pool)
            .await
            .map_err(db)?;
        if done.rows_affected() == 0 {
            Err(StorageError::NotFound)
        } else {
            Ok(())
        }
    }

    async fn set_locale(&self, id: PlayerId, locale: Locale) -> Result<(), StorageError> {
        let done = sqlx::query("UPDATE accounts SET locale = ? WHERE id = ?")
            .bind(locale.code())
            .bind(id_text(id))
            .execute(&self.pool)
            .await
            .map_err(db)?;
        if done.rows_affected() == 0 {
            Err(StorageError::NotFound)
        } else {
            Ok(())
        }
    }
}

async fn insert_seats(
    tx: &mut Transaction<'_, Sqlite>,
    game_id: i64,
    game: &FinishedGame,
) -> Result<(), StorageError> {
    for (player, won) in game.players.iter().zip(game.winners()) {
        let (account, guest) = match &player.seat {
            Seat::Account(id) => {
                let exists = sqlx::query("SELECT 1 FROM accounts WHERE id = ?")
                    .bind(id_text(*id))
                    .fetch_optional(&mut **tx)
                    .await
                    .map_err(db)?;
                if exists.is_none() {
                    return Err(StorageError::NotFound);
                }
                (Some(id_text(*id)), None)
            }
            Seat::Guest(name) => (None, Some(name.clone())),
        };
        sqlx::query("INSERT INTO game_players (game_id, account_id, guest_name, tokens, won) VALUES (?, ?, ?, ?, ?)")
            .bind(game_id)
            .bind(account)
            .bind(guest)
            .bind(i64::from(player.tokens))
            .bind(won)
            .execute(&mut **tx)
            .await
            .map_err(db)?;
    }
    Ok(())
}

#[async_trait]
impl GameResultRepository for SqliteStore {
    async fn record(&self, game: &FinishedGame) -> Result<(), StorageError> {
        check_seats(game)?;
        // One transaction: a failure (e.g. an unknown account) stores nothing.
        let mut tx = self.pool.begin().await.map_err(db)?;
        let game_id: i64 =
            sqlx::query("INSERT INTO games (finished_at, ranked) VALUES (?, ?) RETURNING id")
                .bind(millis(game.finished_at)?)
                .bind(game.is_ranked())
                .fetch_one(&mut *tx)
                .await
                .map_err(db)?
                .try_get("id")
                .map_err(db)?;
        insert_seats(&mut tx, game_id, game).await?;
        tx.commit().await.map_err(db)
    }

    async fn stats(&self, id: PlayerId) -> Result<PlayerStats, StorageError> {
        let row = sqlx::query(
            "SELECT COUNT(*) AS played, COALESCE(SUM(p.won), 0) AS won, COALESCE(SUM(p.tokens), 0) AS tokens \
             FROM game_players p JOIN games g ON g.id = p.game_id \
             WHERE g.ranked = 1 AND p.account_id = ?",
        )
        .bind(id_text(id))
        .fetch_one(&self.pool)
        .await
        .map_err(db)?;
        stats_from(&row)
    }

    async fn leaderboard(
        &self,
        sort: LeaderboardSort,
        limit: u32,
    ) -> Result<Vec<LeaderboardEntry>, StorageError> {
        // One fixed query per sort: no SQL is ever built from input (audit S5).
        let sql = match sort {
            LeaderboardSort::GamesWon => leaderboard_sql!("won"),
            LeaderboardSort::GamesPlayed => leaderboard_sql!("played"),
            LeaderboardSort::TokensWon => leaderboard_sql!("tokens"),
        };
        let rows = sqlx::query(sql)
            .bind(i64::from(limit))
            .fetch_all(&self.pool)
            .await
            .map_err(db)?;
        rows.iter()
            .map(|row| {
                Ok(LeaderboardEntry {
                    username: row.try_get("username").map_err(db)?,
                    stats: stats_from(row)?,
                })
            })
            .collect()
    }
}
