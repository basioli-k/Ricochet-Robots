//! Bidding: each player claims how few moves they need for the target.

use std::fmt;

/// Identifies a player. Opaque to the domain; the server maps accounts and
/// guests to random values so the two kinds can never collide.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct PlayerId(u128);

impl PlayerId {
    #[must_use]
    pub const fn new(raw: u128) -> Self {
        Self(raw)
    }
}

/// Milliseconds on a clock chosen by the caller. The domain never reads the
/// time itself; callers pass it in, which keeps rules deterministic in tests.
/// Callers must use one monotonic clock: earlier values rank bids first.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Timestamp(pub u64);

/// A player's current claim.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Bid {
    pub player: PlayerId,
    pub moves: u8,
    /// When this claim was made; earlier wins ties on `moves`.
    pub placed_at: Timestamp,
}

/// Why a bid was refused.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BidError {
    /// Zero moves would mean the target is already reached.
    ZeroMoves,
    /// A player may only lower their own bid.
    NotLower { current: u8 },
}

impl fmt::Display for BidError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ZeroMoves => f.write_str("a bid needs at least one move"),
            Self::NotLower { current } => write!(f, "bid must be lower than {current}"),
        }
    }
}

impl std::error::Error for BidError {}

/// The bids of one round, at most one per player.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Bids {
    /// In arrival order, which breaks ties between equal timestamps.
    bids: Vec<Bid>,
    /// Time of the first accepted bid; fixed even if that bid is lowered.
    opened_at: Option<Timestamp>,
}

impl Bids {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Records a bid, replacing the player's earlier one. A lowered bid
    /// counts from when it was made.
    ///
    /// # Errors
    /// If `moves` is zero or not lower than the player's current bid.
    pub fn place(&mut self, player: PlayerId, moves: u8, at: Timestamp) -> Result<(), BidError> {
        if moves == 0 {
            return Err(BidError::ZeroMoves);
        }
        if let Some(i) = self.bids.iter().position(|b| b.player == player) {
            let current = self.bids[i].moves;
            if moves >= current {
                return Err(BidError::NotLower { current });
            }
            self.bids.remove(i);
        }
        self.bids.push(Bid {
            player,
            moves,
            placed_at: at,
        });
        self.opened_at.get_or_insert(at);
        Ok(())
    }

    /// When the first bid of the round was made; it opens the bidding window.
    #[must_use]
    pub fn first_bid_at(&self) -> Option<Timestamp> {
        self.opened_at
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.bids.is_empty()
    }

    /// Demonstration order: fewest moves first, then earliest bid, then
    /// arrival order.
    #[must_use]
    pub fn ranking(&self) -> Vec<Bid> {
        let mut ranked = self.bids.clone();
        ranked.sort_by_key(|b| (b.moves, b.placed_at)); // stable: keeps arrival order
        ranked
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ANA: PlayerId = PlayerId::new(1);
    const BOB: PlayerId = PlayerId::new(2);
    const CID: PlayerId = PlayerId::new(3);

    fn order(bids: &Bids) -> Vec<(PlayerId, u8)> {
        bids.ranking().iter().map(|b| (b.player, b.moves)).collect()
    }

    #[test]
    fn rejects_zero_moves_without_opening_the_window() {
        let mut bids = Bids::new();
        assert_eq!(bids.place(ANA, 0, Timestamp(0)), Err(BidError::ZeroMoves));
        assert!(bids.is_empty());
        assert_eq!(bids.first_bid_at(), None);
    }

    #[test]
    fn ranks_by_fewest_moves_then_earliest() {
        let mut bids = Bids::new();
        bids.place(ANA, 7, Timestamp(10)).unwrap();
        bids.place(BOB, 5, Timestamp(20)).unwrap();
        bids.place(CID, 7, Timestamp(5)).unwrap();
        assert_eq!(order(&bids), [(BOB, 5), (CID, 7), (ANA, 7)]);
    }

    #[test]
    fn equal_bids_at_the_same_time_keep_arrival_order() {
        let mut bids = Bids::new();
        bids.place(BOB, 4, Timestamp(1)).unwrap();
        bids.place(ANA, 4, Timestamp(1)).unwrap();
        assert_eq!(order(&bids), [(BOB, 4), (ANA, 4)]);
    }

    #[test]
    fn a_player_may_lower_but_not_raise_or_repeat() {
        let mut bids = Bids::new();
        bids.place(ANA, 6, Timestamp(0)).unwrap();
        assert_eq!(
            bids.place(ANA, 6, Timestamp(1)),
            Err(BidError::NotLower { current: 6 })
        );
        assert_eq!(
            bids.place(ANA, 9, Timestamp(1)),
            Err(BidError::NotLower { current: 6 })
        );
        bids.place(ANA, 4, Timestamp(2)).unwrap();
        assert_eq!(order(&bids), [(ANA, 4)]);
    }

    #[test]
    fn a_lowered_bid_counts_from_when_it_was_made() {
        let mut bids = Bids::new();
        bids.place(ANA, 6, Timestamp(0)).unwrap();
        bids.place(BOB, 5, Timestamp(10)).unwrap();
        bids.place(ANA, 5, Timestamp(20)).unwrap();
        assert_eq!(order(&bids), [(BOB, 5), (ANA, 5)]);
    }

    #[test]
    fn the_first_bid_opens_the_window_even_after_it_is_lowered() {
        let mut bids = Bids::new();
        assert_eq!(bids.first_bid_at(), None);
        bids.place(ANA, 6, Timestamp(100)).unwrap();
        bids.place(BOB, 8, Timestamp(150)).unwrap();
        bids.place(ANA, 5, Timestamp(200)).unwrap();
        assert_eq!(bids.first_bid_at(), Some(Timestamp(100)));
    }
}
