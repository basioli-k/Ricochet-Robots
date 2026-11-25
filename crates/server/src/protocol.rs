//! The WebSocket protocol as JSON: what clients send and what the server
//! answers. The types are exported to TypeScript (`web/src/protocol`) with
//! ts-rs when the tests run, and CI fails if the checked-in copies differ,
//! so the client can't drift from the server.

use domain::{
    BOARD_SIZE, Board, Direction, Phase, PlayerId, Position, RobotColor, Robots, Symbol, Target,
    Timestamp,
};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::rooms::state::{GameView, Member, RoomError, RoomEvent, RoomView};

/// A player's ID on the wire: 32 hex digits (too big for a JS number).
#[must_use]
pub fn player_id(id: PlayerId) -> String {
    format!("{:032x}", id.raw())
}

/// The inverse of [`player_id`]; `None` for anything else.
#[must_use]
pub fn parse_player_id(text: &str) -> Option<PlayerId> {
    // Only the form player_id writes: no sign, no uppercase.
    if text.len() != 32 || !text.bytes().all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f')) {
        return None;
    }
    u128::from_str_radix(text, 16).ok().map(PlayerId::new)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum Robot {
    Red,
    Blue,
    Green,
    Yellow,
}

impl From<RobotColor> for Robot {
    fn from(c: RobotColor) -> Self {
        match c {
            RobotColor::Red => Self::Red,
            RobotColor::Blue => Self::Blue,
            RobotColor::Green => Self::Green,
            RobotColor::Yellow => Self::Yellow,
        }
    }
}

impl From<Robot> for RobotColor {
    fn from(r: Robot) -> Self {
        match r {
            Robot::Red => Self::Red,
            Robot::Blue => Self::Blue,
            Robot::Green => Self::Green,
            Robot::Yellow => Self::Yellow,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum Dir {
    Up,
    Down,
    Left,
    Right,
}

impl From<Direction> for Dir {
    fn from(d: Direction) -> Self {
        match d {
            Direction::Up => Self::Up,
            Direction::Down => Self::Down,
            Direction::Left => Self::Left,
            Direction::Right => Self::Right,
        }
    }
}

impl From<Dir> for Direction {
    fn from(d: Dir) -> Self {
        match d {
            Dir::Up => Self::Up,
            Dir::Down => Self::Down,
            Dir::Left => Self::Left,
            Dir::Right => Self::Right,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum Shape {
    Star,
    Planet,
    Gear,
    Moon,
}

impl From<Symbol> for Shape {
    fn from(s: Symbol) -> Self {
        match s {
            Symbol::Star => Self::Star,
            Symbol::Planet => Self::Planet,
            Symbol::Gear => Self::Gear,
            Symbol::Moon => Self::Moon,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
#[serde(tag = "kind", rename_all = "snake_case")]
#[ts(export, rename = "Target")]
pub enum TargetView {
    Colored { robot: Robot, shape: Shape },
    Vortex,
}

impl From<Target> for TargetView {
    fn from(t: Target) -> Self {
        match t {
            Target::Colored(robot, shape) => Self::Colored {
                robot: robot.into(),
                shape: shape.into(),
            },
            Target::Vortex => Self::Vortex,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
#[ts(export)]
pub struct Pos {
    pub row: u8,
    pub col: u8,
}

impl From<Position> for Pos {
    fn from(p: Position) -> Self {
        Self {
            row: p.row(),
            col: p.col(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
#[ts(export, rename = "Robots")]
pub struct RobotsView {
    pub red: Pos,
    pub blue: Pos,
    pub green: Pos,
    pub yellow: Pos,
}

impl From<&Robots> for RobotsView {
    fn from(r: &Robots) -> Self {
        let at = |c| r.position(c).into();
        Self {
            red: at(RobotColor::Red),
            blue: at(RobotColor::Blue),
            green: at(RobotColor::Green),
            yellow: at(RobotColor::Yellow),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[ts(export, rename = "Player")]
pub struct PlayerView {
    pub id: String,
    pub name: String,
    /// Shown with a guest badge; games with a guest are unranked.
    pub is_guest: bool,
}

impl From<&Member> for PlayerView {
    fn from(m: &Member) -> Self {
        Self {
            id: player_id(m.id),
            name: m.name.clone(),
            is_guest: m.is_guest,
        }
    }
}

/// A wall on `side` of the cell at `pos`. Each wall between two cells is
/// listed from both of them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
#[ts(export)]
pub struct Wall {
    pub pos: Pos,
    pub side: Dir,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
#[ts(export)]
pub struct TargetAt {
    pub pos: Pos,
    pub target: TargetView,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[ts(export, rename = "Board")]
pub struct BoardView {
    pub size: u8,
    pub walls: Vec<Wall>,
    pub blocked: Vec<Pos>,
    pub targets: Vec<TargetAt>,
}

impl From<&Board> for BoardView {
    fn from(b: &Board) -> Self {
        let walls = Position::all()
            .flat_map(|pos| Direction::ALL.into_iter().map(move |dir| (pos, dir)))
            .filter(|&(pos, dir)| b.has_wall(pos, dir))
            .map(|(pos, dir)| Wall {
                pos: pos.into(),
                side: dir.into(),
            })
            .collect();
        Self {
            size: BOARD_SIZE,
            walls,
            blocked: Position::all()
                .filter(|&p| b.is_blocked(p))
                .map(Pos::from)
                .collect(),
            targets: b
                .targets()
                .iter()
                .map(|&(pos, target)| TargetAt {
                    pos: pos.into(),
                    target: target.into(),
                })
                .collect(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(tag = "kind", rename_all = "snake_case")]
#[ts(export, rename = "Phase")]
pub enum PhaseView {
    Bidding,
    Demonstrating {
        player: String,
        bid: u8,
        moves_made: u8,
    },
    Solved {
        winner: String,
        moves: u8,
    },
    Unsolved,
}

impl From<Phase> for PhaseView {
    fn from(p: Phase) -> Self {
        match p {
            Phase::Bidding => Self::Bidding,
            Phase::Demonstrating {
                player,
                bid,
                moves_made,
            } => Self::Demonstrating {
                player: player_id(player),
                bid,
                moves_made,
            },
            Phase::Solved { winner, moves } => Self::Solved {
                winner: player_id(winner),
                moves,
            },
            Phase::Unsolved => Self::Unsolved,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[ts(export)]
pub struct BidView {
    pub player: String,
    pub moves: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[ts(export)]
pub struct Score {
    pub player: String,
    pub tokens: u32,
}

fn scores(scores: &[(PlayerId, u32)]) -> Vec<Score> {
    scores
        .iter()
        .map(|&(p, tokens)| Score {
            player: player_id(p),
            tokens,
        })
        .collect()
}

/// Milliseconds since the Unix epoch; safe as a JS number until year 287396.
fn ms(t: Timestamp) -> u64 {
    t.0
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[ts(export)]
pub struct GameSnapshot {
    pub target: TargetView,
    pub robots: RobotsView,
    pub phase: PhaseView,
    /// Best first.
    pub bids: Vec<BidView>,
    /// When the current stage ends on its own (ms since the epoch).
    #[ts(type = "number | null")]
    pub deadline: Option<u64>,
    pub scores: Vec<Score>,
    pub targets_left: u32,
    pub over: bool,
}

impl From<&GameView> for GameSnapshot {
    fn from(g: &GameView) -> Self {
        Self {
            target: g.target.into(),
            robots: (&g.robots).into(),
            phase: g.phase.into(),
            bids: g
                .bids
                .iter()
                .map(|b| BidView {
                    player: player_id(b.player),
                    moves: b.moves,
                })
                .collect(),
            deadline: g.deadline.map(ms),
            scores: scores(&g.scores),
            targets_left: u32::try_from(g.targets_left).unwrap_or(u32::MAX),
            over: g.over,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[ts(export)]
pub struct RoomSnapshot {
    pub members: Vec<PlayerView>,
    pub board: BoardView,
    pub game: Option<GameSnapshot>,
}

impl From<&RoomView> for RoomSnapshot {
    fn from(v: &RoomView) -> Self {
        Self {
            members: v.members.iter().map(PlayerView::from).collect(),
            board: (&v.board).into(),
            game: v.game.as_ref().map(GameSnapshot::from),
        }
    }
}

/// What a client may send. The player is always the session's, never
/// part of the message.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, TS)]
#[serde(tag = "type", rename_all = "snake_case")]
#[ts(export)]
pub enum ClientMessage {
    Start,
    Bid {
        moves: u8,
    },
    Move {
        robot: Robot,
        dir: Dir,
    },
    Concede,
    Chat {
        text: String,
    },
    /// Leaves through this socket; a player with other sockets open in
    /// the room stays a member until the last one leaves or closes.
    Leave,
}

/// What the server sends: a snapshot on connecting, then each change after
/// it, none repeated. An `error` with code `lagged` means the socket fell
/// behind and is closing: reconnect for a fresh snapshot.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(tag = "type", rename_all = "snake_case")]
#[ts(export)]
pub enum ServerMessage {
    Snapshot {
        /// The receiving player's ID.
        you: String,
        room: RoomSnapshot,
    },
    Joined {
        player: PlayerView,
    },
    Left {
        player: String,
    },
    RoundStarted {
        target: TargetView,
        robots: RobotsView,
    },
    BidPlaced {
        player: String,
        moves: u8,
        #[ts(type = "number")]
        closes_at: u64,
    },
    DemonstrationStarted {
        player: String,
        bid: u8,
        #[ts(type = "number")]
        until: u64,
        /// Back at the round's start if an earlier demonstrator failed.
        robots: RobotsView,
    },
    RobotMoved {
        robot: Robot,
        robots: RobotsView,
    },
    RoundResolved {
        phase: PhaseView,
        /// Where the robots ended: as solved, or back at the start.
        robots: RobotsView,
    },
    GameOver {
        leaders: Vec<String>,
        scores: Vec<Score>,
    },
    Chat {
        player: String,
        text: String,
    },
    /// A refused message, by stable code for the client to translate.
    Error {
        code: String,
    },
}

impl ServerMessage {
    #[must_use]
    pub fn error(code: &str) -> Self {
        Self::Error { code: code.into() }
    }
}

impl From<RoomEvent> for ServerMessage {
    fn from(e: RoomEvent) -> Self {
        match e {
            RoomEvent::Joined(m) => Self::Joined {
                player: (&m).into(),
            },
            RoomEvent::Left(p) => Self::Left {
                player: player_id(p),
            },
            RoomEvent::RoundStarted { target, robots } => Self::RoundStarted {
                target: target.into(),
                robots: (&robots).into(),
            },
            RoomEvent::BidPlaced {
                player,
                moves,
                closes_at,
            } => Self::BidPlaced {
                player: player_id(player),
                moves,
                closes_at: ms(closes_at),
            },
            RoomEvent::Demonstrating {
                player,
                bid,
                until,
                robots,
            } => Self::DemonstrationStarted {
                player: player_id(player),
                bid,
                until: ms(until),
                robots: (&robots).into(),
            },
            RoomEvent::RobotMoved { robot, robots } => Self::RobotMoved {
                robot: robot.into(),
                robots: (&robots).into(),
            },
            RoomEvent::RoundEnded { phase, robots } => Self::RoundResolved {
                phase: phase.into(),
                robots: (&robots).into(),
            },
            RoomEvent::GameOver { leaders, scores: s } => Self::GameOver {
                leaders: leaders.into_iter().map(player_id).collect(),
                scores: scores(&s),
            },
            RoomEvent::Chat { player, text } => Self::Chat {
                player: player_id(player),
                text,
            },
        }
    }
}

/// The stable code for a refused room command.
#[must_use]
pub const fn room_error_code(e: RoomError) -> &'static str {
    use domain::{BidError, GameError, RoundError};
    match e {
        RoomError::Full => "room_full",
        RoomError::NameTaken => "name_taken",
        RoomError::NotMember => "not_member",
        RoomError::AlreadyStarted => "already_started",
        RoomError::NotStarted => "not_started",
        RoomError::BadChat => "bad_chat",
        RoomError::Game(GameError::GameOver) => "game_over",
        RoomError::Game(GameError::RoundInProgress) => "round_in_progress",
        RoomError::Game(GameError::Round(r)) => match r {
            RoundError::WrongPhase => "wrong_phase",
            RoundError::BiddingClosed => "bidding_closed",
            RoundError::BiddingStillOpen => "bidding_open",
            RoundError::NoBids => "no_bids",
            RoundError::NotYourTurn => "not_your_turn",
            RoundError::RobotDidNotMove => "robot_did_not_move",
            RoundError::Bid(BidError::ZeroMoves) => "bid_zero",
            RoundError::Bid(BidError::NotLower { .. }) => "bid_not_lower",
        },
    }
}
