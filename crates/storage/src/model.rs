//! What storage keeps: accounts and finished games.

use domain::{PlayerId, Timestamp};

/// The language an account reads emails and the UI in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Locale {
    En,
    Hr,
}

impl Locale {
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::En => "en",
            Self::Hr => "hr",
        }
    }

    #[must_use]
    pub fn from_code(code: &str) -> Option<Self> {
        match code {
            "en" => Some(Self::En),
            "hr" => Some(Self::Hr),
            _ => None,
        }
    }
}

/// A registered player. The id doubles as their in-game `PlayerId`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Account {
    pub id: PlayerId,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub locale: Locale,
    pub created_at: Timestamp,
    /// When the email address was confirmed; unconfirmed accounts can't log in.
    pub confirmed_at: Option<Timestamp>,
}

/// One seat in a finished game.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Seat {
    Account(PlayerId),
    /// Guests are stored by nickname only and never linked to an account.
    Guest(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Participant {
    pub seat: Seat,
    pub tokens: u32,
}

/// A game to record once it has ended.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FinishedGame {
    pub finished_at: Timestamp,
    pub players: Vec<Participant>,
}

impl FinishedGame {
    /// Ranked games count towards stats: every player has an account and
    /// there are at least two, so guests can't be used to farm wins.
    #[must_use]
    pub fn is_ranked(&self) -> bool {
        self.players.len() >= 2
            && self
                .players
                .iter()
                .all(|p| matches!(p.seat, Seat::Account(_)))
    }

    /// Whether each player (in order) won: the top score wins, ties all win,
    /// and nobody wins a game where nobody scored.
    #[must_use]
    pub fn winners(&self) -> Vec<bool> {
        let best = self.players.iter().map(|p| p.tokens).max().unwrap_or(0);
        self.players
            .iter()
            .map(|p| best > 0 && p.tokens == best)
            .collect()
    }
}

/// An account's totals over ranked games.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct PlayerStats {
    pub games_played: u64,
    pub games_won: u64,
    pub tokens_won: u64,
}

/// Leaderboard order; a closed set, so it can never carry SQL (audit S5).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LeaderboardSort {
    GamesWon,
    GamesPlayed,
    TokensWon,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LeaderboardEntry {
    pub username: String,
    pub stats: PlayerStats,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn game(players: &[(Seat, u32)]) -> FinishedGame {
        FinishedGame {
            finished_at: Timestamp(0),
            players: players
                .iter()
                .map(|(seat, tokens)| Participant {
                    seat: seat.clone(),
                    tokens: *tokens,
                })
                .collect(),
        }
    }

    const ANA: Seat = Seat::Account(PlayerId::new(1));
    const BOB: Seat = Seat::Account(PlayerId::new(2));

    fn guest() -> Seat {
        Seat::Guest("visitor".into())
    }

    #[test]
    fn ranked_needs_two_or_more_accounts_and_no_guests() {
        assert!(game(&[(ANA, 3), (BOB, 1)]).is_ranked());
        assert!(!game(&[(ANA, 3)]).is_ranked());
        assert!(!game(&[(ANA, 3), (BOB, 1), (guest(), 2)]).is_ranked());
        assert!(!game(&[]).is_ranked());
    }

    #[test]
    fn the_top_score_wins_and_ties_all_win() {
        assert_eq!(game(&[(ANA, 3), (BOB, 1)]).winners(), [true, false]);
        assert_eq!(
            game(&[(ANA, 2), (BOB, 2), (guest(), 1)]).winners(),
            [true, true, false]
        );
    }

    #[test]
    fn nobody_wins_when_nobody_scored() {
        assert_eq!(game(&[(ANA, 0), (BOB, 0)]).winners(), [false, false]);
    }

    #[test]
    fn locale_codes_round_trip() {
        for locale in [Locale::En, Locale::Hr] {
            assert_eq!(Locale::from_code(locale.code()), Some(locale));
        }
        assert_eq!(Locale::from_code("de"), None);
    }
}
