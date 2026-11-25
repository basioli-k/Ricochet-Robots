//! One room's rules: who is in it and its game, as a synchronous state
//! machine. The async actor around it only forwards commands and events, so
//! everything here is tested without a runtime. Time arrives as `now`.

use std::collections::HashMap;

use storage::{FinishedGame, Participant, Seat};

use domain::{
    Bid, Board, Direction, Game, GameError, Phase, PlayerId, RobotColor, Robots, Shuffle, Target,
    Timestamp,
};

/// How long bidding stays open after the first bid.
pub const BIDDING_WINDOW_MS: u64 = 60_000;
/// How long a demonstrator may take before conceding automatically.
pub const DEMONSTRATION_MS: u64 = 60_000;
/// A round nobody bids on is abandoned after this long.
pub const IDLE_ROUND_MS: u64 = 3 * 60_000;
/// Pause after a round ends so players see the result before the next.
pub const RESULT_PAUSE_MS: u64 = 5_000;
/// Longest chat message, in characters, after trimming.
pub const CHAT_MAX_CHARS: usize = 200;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Member {
    pub id: PlayerId,
    pub name: String,
    pub is_guest: bool,
}

/// What changed, for broadcasting to the room's members.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RoomEvent {
    Joined(Member),
    Left(PlayerId),
    RoundStarted {
        target: Target,
        robots: Robots,
    },
    BidPlaced {
        player: PlayerId,
        moves: u8,
        closes_at: Timestamp,
    },
    Demonstrating {
        player: PlayerId,
        bid: u8,
        /// When they concede automatically.
        until: Timestamp,
    },
    RobotMoved {
        robot: RobotColor,
        robots: Robots,
    },
    RoundEnded {
        phase: Phase,
    },
    GameOver {
        leaders: Vec<PlayerId>,
        /// Tokens won, for every player who won any.
        scores: Vec<(PlayerId, u32)>,
    },
    Chat {
        player: PlayerId,
        text: String,
    },
}

/// Everything a member needs to draw the room, e.g. after (re)connecting.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoomView {
    pub members: Vec<Member>,
    pub board: Board,
    pub game: Option<GameView>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameView {
    pub target: Target,
    pub robots: Robots,
    pub phase: Phase,
    pub bids: Vec<Bid>,
    /// When the current stage ends on its own, if it does.
    pub deadline: Option<Timestamp>,
    pub scores: Vec<(PlayerId, u32)>,
    pub targets_left: usize,
    pub over: bool,
}

/// Why a command was refused; the room is unchanged when this is returned.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoomError {
    Full,
    NameTaken,
    NotMember,
    AlreadyStarted,
    NotStarted,
    /// Empty, too long, or containing control characters.
    BadChat,
    Game(GameError),
}

impl From<GameError> for RoomError {
    fn from(e: GameError) -> Self {
        Self::Game(e)
    }
}

/// When the current round entered its stage (see [`stage`]), to time it out.
#[derive(Debug, Clone, Copy)]
struct Since {
    phase: Phase,
    at: Timestamp,
}

/// A room plays one game; there is no rematch yet.
#[derive(Debug)]
pub struct RoomState {
    capacity: usize,
    board: Board,
    members: Vec<Member>,
    /// Everyone who was a member while the game ran, for its result.
    players: Vec<Member>,
    /// Open connections per member; always one entry per member.
    connections: HashMap<PlayerId, usize>,
    setup: Option<(Board, Robots)>,
    game: Option<Game>,
    since: Option<Since>,
}

type Events = Result<Vec<RoomEvent>, RoomError>;

impl RoomState {
    /// A room for up to `capacity` members that will play `board` from
    /// `robots` once started.
    #[must_use]
    pub fn new(board: Board, robots: Robots, capacity: usize) -> Self {
        Self {
            capacity,
            board: board.clone(),
            members: Vec::new(),
            players: Vec::new(),
            connections: HashMap::new(),
            setup: Some((board, robots)),
            game: None,
            since: None,
        }
    }

    #[must_use]
    pub fn members(&self) -> &[Member] {
        &self.members
    }

    #[must_use]
    pub const fn game(&self) -> Option<&Game> {
        self.game.as_ref()
    }

    #[must_use]
    pub fn view(&self) -> RoomView {
        RoomView {
            members: self.members.clone(),
            board: self.board.clone(),
            game: self.game.as_ref().map(|game| GameView {
                target: game.round().target(),
                robots: *game.round().robots(),
                phase: game.round().phase(),
                bids: game.round().bids(),
                deadline: self.deadline(),
                scores: scores(game),
                targets_left: game.targets_left(),
                over: game.is_over(),
            }),
        }
    }

    /// The finished game to record: everyone who played while it ran, with
    /// the tokens they won. `None` until the game is over.
    #[must_use]
    pub fn result(&self, now: Timestamp) -> Option<FinishedGame> {
        let game = self.game.as_ref().filter(|g| g.is_over())?;
        let players = self
            .players
            .iter()
            .map(|m| Participant {
                seat: if m.is_guest {
                    Seat::Guest(m.name.clone())
                } else {
                    Seat::Account(m.id)
                },
                tokens: game.scores().get(&m.id).copied().unwrap_or(0),
            })
            .collect();
        Some(FinishedGame {
            finished_at: now,
            players,
        })
    }

    fn add_player(&mut self, member: &Member) {
        if !self.players.iter().any(|p| p.id == member.id) {
            self.players.push(member.clone());
        }
    }

    /// When the current stage ends on its own: bidding closing, a
    /// demonstrator timing out, or the next round after the result pause.
    fn deadline(&self) -> Option<Timestamp> {
        let game = self.game.as_ref().filter(|g| !g.is_over())?;
        let since = self.since?;
        let after = |ms: u64| Some(Timestamp(since.at.0.saturating_add(ms)));
        match game.round().phase() {
            Phase::Bidding => game.round().bidding_closes_at(),
            Phase::Demonstrating { .. } => after(DEMONSTRATION_MS),
            Phase::Solved { .. } | Phase::Unsolved => after(RESULT_PAUSE_MS),
        }
    }

    /// Posts a chat message, trimmed.
    ///
    /// # Errors
    /// `NotMember`, or `BadChat` for an empty or overlong message or one
    /// with control characters.
    pub fn chat(&mut self, player: PlayerId, text: &str) -> Events {
        self.require_member(player)?;
        let text = text.trim();
        if text.is_empty()
            || text.chars().count() > CHAT_MAX_CHARS
            || text.chars().any(char::is_control)
        {
            return Err(RoomError::BadChat);
        }
        Ok(vec![RoomEvent::Chat {
            player,
            text: text.to_owned(),
        }])
    }

    /// Adds a member; names are unique in the room ignoring case. Each
    /// join is one connection: joining again (a second tab, or reconnecting
    /// before the old socket is noticed gone) adds a connection and no
    /// event, and the member stays until every connection has left.
    ///
    /// # Errors
    /// `Full` or `NameTaken`.
    pub fn join(&mut self, member: Member) -> Events {
        if let Some(n) = self.connections.get_mut(&member.id) {
            *n += 1;
            return Ok(Vec::new());
        }
        if self.members.len() >= self.capacity {
            return Err(RoomError::Full);
        }
        if self
            .members
            .iter()
            .any(|m| m.name.eq_ignore_ascii_case(&member.name))
        {
            return Err(RoomError::NameTaken);
        }
        if self.game.is_some() {
            self.add_player(&member);
        }
        self.connections.insert(member.id, 1);
        self.members.push(member.clone());
        Ok(vec![RoomEvent::Joined(member)])
    }

    /// Ends one of a member's connections; with the last one the member
    /// leaves, and a demonstrator who leaves concedes.
    ///
    /// # Errors
    /// `NotMember`.
    pub fn leave(&mut self, id: PlayerId, now: Timestamp) -> Events {
        let connections = self.connections.get_mut(&id).ok_or(RoomError::NotMember)?;
        *connections -= 1;
        if *connections > 0 {
            return Ok(Vec::new());
        }
        self.connections.remove(&id);
        let index = self
            .members
            .iter()
            .position(|m| m.id == id)
            .ok_or(RoomError::NotMember)?;
        self.members.remove(index);
        let mut events = vec![RoomEvent::Left(id)];
        if let Some(game) = &mut self.game
            && matches!(game.round().phase(), Phase::Demonstrating { player, .. } if player == id)
        {
            game.concede(id)?;
            events.extend(self.after_change(now));
        }
        Ok(events)
    }

    /// Starts the game; any member may.
    ///
    /// # Errors
    /// `NotMember` or `AlreadyStarted`.
    pub fn start(&mut self, by: PlayerId, shuffle: &mut impl Shuffle, now: Timestamp) -> Events {
        self.require_member(by)?;
        let (board, robots) = self.setup.take().ok_or(RoomError::AlreadyStarted)?;
        let Some(game) = Game::new(board, robots, BIDDING_WINDOW_MS, shuffle) else {
            return Ok(vec![RoomEvent::GameOver {
                leaders: Vec::new(),
                scores: Vec::new(),
            }]);
        };
        let event = round_started(&game);
        self.game = Some(game);
        self.players.clone_from(&self.members);
        self.mark(now);
        Ok(vec![event])
    }

    /// # Errors
    /// `NotMember`, `NotStarted`, or the game's refusal.
    pub fn bid(&mut self, player: PlayerId, moves: u8, now: Timestamp) -> Events {
        self.require_member(player)?;
        let game = self.game.as_mut().ok_or(RoomError::NotStarted)?;
        game.bid(player, moves, now)?;
        let closes_at = game.round().bidding_closes_at().unwrap_or(now);
        Ok(vec![RoomEvent::BidPlaced {
            player,
            moves,
            closes_at,
        }])
    }

    /// # Errors
    /// `NotMember`, `NotStarted`, or the game's refusal.
    pub fn move_robot(
        &mut self,
        player: PlayerId,
        robot: RobotColor,
        dir: Direction,
        now: Timestamp,
    ) -> Events {
        self.require_member(player)?;
        let game = self.game.as_mut().ok_or(RoomError::NotStarted)?;
        game.move_robot(player, robot, dir)?;
        let mut events = vec![RoomEvent::RobotMoved {
            robot,
            robots: *game.round().robots(),
        }];
        events.extend(self.after_change(now));
        Ok(events)
    }

    /// # Errors
    /// `NotMember`, `NotStarted`, or the game's refusal.
    pub fn concede(&mut self, player: PlayerId, now: Timestamp) -> Events {
        self.require_member(player)?;
        self.game
            .as_mut()
            .ok_or(RoomError::NotStarted)?
            .concede(player)?;
        Ok(self.after_change(now))
    }

    /// Advances anything that is due by `now`: closing bidding, timing out
    /// a demonstrator, abandoning an unbid round, or dealing the next round
    /// after the result pause.
    pub fn tick(&mut self, now: Timestamp) -> Vec<RoomEvent> {
        let (Some(game), Some(since)) = (self.game.as_mut(), self.since) else {
            return Vec::new();
        };
        if game.is_over() {
            return Vec::new();
        }
        let elapsed = now.0.saturating_sub(since.at.0);
        match game.round().phase() {
            Phase::Bidding => {
                let due = game.round().bidding_closes_at();
                if due.is_some_and(|t| now >= t) {
                    let _ = game.close_bidding(now);
                    return self.after_change(now);
                }
                if due.is_none() && elapsed >= IDLE_ROUND_MS {
                    let _ = game.abandon_round();
                    return self.after_change(now);
                }
                Vec::new()
            }
            Phase::Demonstrating { player, .. } if elapsed >= DEMONSTRATION_MS => {
                let _ = game.concede(player);
                self.after_change(now)
            }
            Phase::Solved { .. } | Phase::Unsolved if elapsed >= RESULT_PAUSE_MS => {
                let _ = game.next_round();
                if game.is_over() {
                    vec![RoomEvent::GameOver {
                        leaders: game.leaders(),
                        scores: scores(game),
                    }]
                } else {
                    let event = round_started(game);
                    self.mark(now);
                    vec![event]
                }
            }
            _ => Vec::new(),
        }
    }

    /// Events for a phase change, restarting the phase clock if it changed.
    /// A bidder who has left can't demonstrate, so their turn is skipped.
    fn after_change(&mut self, now: Timestamp) -> Vec<RoomEvent> {
        let Some(game) = &mut self.game else {
            return Vec::new();
        };
        while let Phase::Demonstrating { player, .. } = game.round().phase()
            && !self.members.iter().any(|m| m.id == player)
        {
            if game.concede(player).is_err() {
                break;
            }
        }
        let phase = game.round().phase();
        if self.since.is_some_and(|s| s.phase == stage(phase)) {
            return Vec::new();
        }
        self.mark(now);
        match phase {
            Phase::Demonstrating { player, bid, .. } => vec![RoomEvent::Demonstrating {
                player,
                bid,
                until: Timestamp(now.0.saturating_add(DEMONSTRATION_MS)),
            }],
            Phase::Solved { .. } | Phase::Unsolved => vec![RoomEvent::RoundEnded { phase }],
            Phase::Bidding => Vec::new(),
        }
    }

    fn mark(&mut self, now: Timestamp) {
        self.since = self.game.as_ref().map(|g| Since {
            phase: stage(g.round().phase()),
            at: now,
        });
    }

    fn require_member(&self, id: PlayerId) -> Result<(), RoomError> {
        self.members
            .iter()
            .any(|m| m.id == id)
            .then_some(())
            .ok_or(RoomError::NotMember)
    }
}

/// The phase without its move count, so a demonstrator's moves don't
/// restart their clock.
const fn stage(phase: Phase) -> Phase {
    match phase {
        Phase::Demonstrating { player, bid, .. } => Phase::Demonstrating {
            player,
            bid,
            moves_made: 0,
        },
        other => other,
    }
}

fn scores(game: &Game) -> Vec<(PlayerId, u32)> {
    game.scores().iter().map(|(&p, &s)| (p, s)).collect()
}

fn round_started(game: &Game) -> RoomEvent {
    RoomEvent::RoundStarted {
        target: game.round().target(),
        robots: *game.round().robots(),
    }
}
