//! A whole game: one round per target until the deck is empty.

use std::collections::BTreeMap;

use crate::bidding::{PlayerId, Timestamp};
use crate::board::Board;
use crate::geometry::Direction;
use crate::pieces::{RobotColor, Target};
use crate::robots::Robots;
use crate::round::{Phase, Round, RoundError, completes};

/// Orders the target deck. Injected so games are reproducible in tests; the
/// server supplies a random implementation.
pub trait Shuffle {
    /// Reorders `deck` in place. Must only permute it: no entry may be added,
    /// dropped or duplicated (checked in debug builds).
    fn shuffle(&mut self, deck: &mut [Target]);
}

/// Why a game action was refused. The game is unchanged when this is returned.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameError {
    /// The current round is still being played.
    RoundInProgress,
    /// Every target has been played.
    GameOver,
    Round(RoundError),
}

impl std::fmt::Display for GameError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Round(e) => e.fmt(f),
            other => write!(f, "{other:?}"),
        }
    }
}

impl std::error::Error for GameError {}

impl From<RoundError> for GameError {
    fn from(e: RoundError) -> Self {
        Self::Round(e)
    }
}

/// Plays each target on the board at most once, in shuffled order. Robots carry
/// over between rounds. Each solved target scores one token for its winner.
///
/// A target that the robots already complete when it comes up would be won
/// by any move, so it goes to the bottom of the deck instead; the game ends
/// early if no remaining target is playable.
///
/// `Game` is the intended entry point for play: it keeps the board, the
/// deck and the current round consistent.
#[derive(Debug, Clone)]
pub struct Game {
    board: Board,
    window_ms: u64,
    /// Targets still to play; the next one is last.
    deck: Vec<Target>,
    round: Round,
    scores: BTreeMap<PlayerId, u32>,
    over: bool,
}

impl Game {
    /// Starts the first round. Returns `None` if no target is playable (the
    /// board has none, or the robots already complete every one).
    #[must_use]
    pub fn new(
        board: Board,
        robots: Robots,
        window_ms: u64,
        shuffle: &mut impl Shuffle,
    ) -> Option<Self> {
        let mut deck: Vec<Target> = board.targets().iter().map(|(_, t)| *t).collect();
        let before = deck.clone();
        shuffle.shuffle(&mut deck);
        debug_assert!(
            is_permutation(&before, &deck),
            "Shuffle must only reorder the deck"
        );
        deck.reverse(); // so pop() follows the shuffled order
        let first = draw(&board, &mut deck, robots)?;
        Some(Self {
            board,
            window_ms,
            deck,
            round: Round::new(first, robots, window_ms),
            scores: BTreeMap::new(),
            over: false,
        })
    }

    #[must_use]
    pub const fn board(&self) -> &Board {
        &self.board
    }

    #[must_use]
    pub const fn round(&self) -> &Round {
        &self.round
    }

    /// Targets left after the current one; 0 once the game is over, even if
    /// it ended early with unplayable targets.
    #[must_use]
    pub fn targets_left(&self) -> usize {
        if self.over { 0 } else { self.deck.len() }
    }

    #[must_use]
    pub const fn is_over(&self) -> bool {
        self.over
    }

    /// Tokens won so far, for players who have won any.
    #[must_use]
    pub const fn scores(&self) -> &BTreeMap<PlayerId, u32> {
        &self.scores
    }

    /// Players with the most tokens (several on a tie; none if nobody scored).
    #[must_use]
    pub fn leaders(&self) -> Vec<PlayerId> {
        // Only winners are in `scores`, so every entry has at least one token.
        let best = self.scores.values().max();
        self.scores
            .iter()
            .filter(|(_, s)| Some(*s) == best)
            .map(|(p, _)| *p)
            .collect()
    }

    /// # Errors
    /// See [`Round::bid`].
    pub fn bid(&mut self, player: PlayerId, moves: u8, at: Timestamp) -> Result<(), GameError> {
        Ok(self.round.bid(player, moves, at)?)
    }

    /// # Errors
    /// See [`Round::close_bidding`].
    pub fn close_bidding(&mut self, now: Timestamp) -> Result<(), GameError> {
        Ok(self.round.close_bidding(now)?)
    }

    /// # Errors
    /// See [`Round::abandon`].
    pub fn abandon_round(&mut self) -> Result<(), GameError> {
        Ok(self.round.abandon()?)
    }

    /// # Errors
    /// See [`Round::move_robot`].
    pub fn move_robot(
        &mut self,
        player: PlayerId,
        robot: RobotColor,
        dir: Direction,
    ) -> Result<(), GameError> {
        Ok(self.round.move_robot(&self.board, player, robot, dir)?)
    }

    /// # Errors
    /// See [`Round::concede`].
    pub fn concede(&mut self, player: PlayerId) -> Result<(), GameError> {
        Ok(self.round.concede(player)?)
    }

    /// Scores the finished round and starts the next one, or ends the game
    /// when the deck is empty.
    ///
    /// # Errors
    /// If the round is still in play or the game is already over.
    pub fn next_round(&mut self) -> Result<(), GameError> {
        if self.over {
            return Err(GameError::GameOver);
        }
        match self.round.phase() {
            Phase::Solved { winner, .. } => *self.scores.entry(winner).or_insert(0) += 1,
            Phase::Unsolved => {}
            Phase::Bidding | Phase::Demonstrating { .. } => return Err(GameError::RoundInProgress),
        }
        let robots = *self.round.robots();
        match draw(&self.board, &mut self.deck, robots) {
            Some(target) => self.round = Round::new(target, robots, self.window_ms),
            None => self.over = true,
        }
        Ok(())
    }
}

/// Takes the next target the robots don't already complete, moving any
/// that they do to the bottom of the deck. `None` if none is playable.
fn draw(board: &Board, deck: &mut Vec<Target>, robots: Robots) -> Option<Target> {
    for _ in 0..deck.len() {
        let target = deck.pop()?;
        if !completes(board, target, robots) {
            return Some(target);
        }
        deck.insert(0, target);
    }
    None
}

fn is_permutation(a: &[Target], b: &[Target]) -> bool {
    let (mut a, mut b) = (a.to_vec(), b.to_vec());
    a.sort_unstable();
    b.sort_unstable();
    a == b
}
