//! One task per room owns its `RoomState`: commands come in over a channel,
//! each is answered, and the resulting events are broadcast. Nothing about
//! a room is shared or locked.

use std::sync::Arc;
use std::time::Duration;

use domain::{Board, Direction, PlayerId, RobotColor, Robots, Shuffle, Target};
use tokio::sync::{broadcast, mpsc, oneshot};

use super::state::{Member, RoomError, RoomEvent, RoomState};
use crate::clock::Clock;

/// How often a room checks its timers.
pub const TICK: Duration = Duration::from_millis(250);
/// Events kept for slow subscribers before they start missing some.
const EVENT_BUFFER: usize = 256;

/// Orders a target deck; random in production, fixed in tests.
pub trait DeckShuffler: Send + Sync {
    fn shuffle(&self, deck: &mut [Target]);
}

/// Shuffles with the thread's cryptographically secure RNG.
#[derive(Debug, Default)]
pub struct RandomShuffler;

impl DeckShuffler for RandomShuffler {
    fn shuffle(&self, deck: &mut [Target]) {
        use rand::seq::SliceRandom;
        deck.shuffle(&mut rand::rng());
    }
}

/// Adapts a shared `DeckShuffler` to the domain's `Shuffle`.
struct With<'a>(&'a dyn DeckShuffler);

impl Shuffle for With<'_> {
    fn shuffle(&mut self, deck: &mut [Target]) {
        self.0.shuffle(deck);
    }
}

#[derive(Debug)]
enum Command {
    Join(Member),
    Leave(PlayerId),
    Start(PlayerId),
    Bid(PlayerId, u8),
    Move(PlayerId, RobotColor, Direction),
    Concede(PlayerId),
    Tick,
    Members,
}

enum Reply {
    Events(Result<(), RoomError>),
    Members(Vec<Member>),
}

/// Why a call on a room failed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CallError {
    Room(RoomError),
    /// The room's task has stopped (the room was closed).
    Closed,
}

impl From<RoomError> for CallError {
    fn from(e: RoomError) -> Self {
        Self::Room(e)
    }
}

/// A cheap, cloneable handle to a running room.
#[derive(Clone)]
pub struct RoomHandle {
    commands: mpsc::Sender<(Command, oneshot::Sender<Reply>)>,
    events: broadcast::Sender<RoomEvent>,
}

impl std::fmt::Debug for RoomHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RoomHandle").finish_non_exhaustive()
    }
}

impl RoomHandle {
    /// Starts a room's task on the current runtime. It ticks every [`TICK`]
    /// and stops once every handle is dropped.
    #[must_use]
    pub fn spawn(
        board: Board,
        robots: Robots,
        capacity: usize,
        clock: Arc<dyn Clock>,
        shuffler: Arc<dyn DeckShuffler>,
    ) -> Self {
        let (commands, mut inbox) = mpsc::channel::<(Command, oneshot::Sender<Reply>)>(64);
        let (events, _) = broadcast::channel(EVENT_BUFFER);
        let out = events.clone();
        tokio::spawn(async move {
            let mut room = RoomState::new(board, robots, capacity);
            let mut ticker = tokio::time::interval(TICK);
            loop {
                tokio::select! {
                    message = inbox.recv() => {
                        let Some((command, reply)) = message else { break };
                        let answer = apply(&mut room, command, clock.now(), shuffler.as_ref(), &out);
                        let _ = reply.send(answer);
                    }
                    _ = ticker.tick() => publish(&out, room.tick(clock.now())),
                }
            }
        });
        Self { commands, events }
    }

    /// A stream of everything that happens in the room from now on.
    #[must_use]
    pub fn subscribe(&self) -> broadcast::Receiver<RoomEvent> {
        self.events.subscribe()
    }

    async fn call(&self, command: Command) -> Result<Reply, CallError> {
        let (reply, answer) = oneshot::channel();
        self.commands
            .send((command, reply))
            .await
            .map_err(|_| CallError::Closed)?;
        answer.await.map_err(|_| CallError::Closed)
    }

    async fn act(&self, command: Command) -> Result<(), CallError> {
        match self.call(command).await? {
            Reply::Events(result) => Ok(result?),
            Reply::Members(_) => unreachable!("only Members answers with members"),
        }
    }

    /// # Errors
    /// The room's refusal, or `Closed`.
    pub async fn join(&self, member: Member) -> Result<(), CallError> {
        self.act(Command::Join(member)).await
    }

    /// # Errors
    /// The room's refusal, or `Closed`.
    pub async fn leave(&self, player: PlayerId) -> Result<(), CallError> {
        self.act(Command::Leave(player)).await
    }

    /// # Errors
    /// The room's refusal, or `Closed`.
    pub async fn start(&self, player: PlayerId) -> Result<(), CallError> {
        self.act(Command::Start(player)).await
    }

    /// # Errors
    /// The room's refusal, or `Closed`.
    pub async fn bid(&self, player: PlayerId, moves: u8) -> Result<(), CallError> {
        self.act(Command::Bid(player, moves)).await
    }

    /// # Errors
    /// The room's refusal, or `Closed`.
    pub async fn move_robot(
        &self,
        player: PlayerId,
        robot: RobotColor,
        dir: Direction,
    ) -> Result<(), CallError> {
        self.act(Command::Move(player, robot, dir)).await
    }

    /// # Errors
    /// The room's refusal, or `Closed`.
    pub async fn concede(&self, player: PlayerId) -> Result<(), CallError> {
        self.act(Command::Concede(player)).await
    }

    /// Checks timers now instead of waiting for the next tick (for tests).
    ///
    /// # Errors
    /// `Closed`.
    pub async fn tick(&self) -> Result<(), CallError> {
        self.act(Command::Tick).await
    }

    /// # Errors
    /// `Closed`.
    pub async fn members(&self) -> Result<Vec<Member>, CallError> {
        match self.call(Command::Members).await? {
            Reply::Members(members) => Ok(members),
            Reply::Events(_) => unreachable!("Members answers with members"),
        }
    }
}

fn apply(
    room: &mut RoomState,
    command: Command,
    now: domain::Timestamp,
    shuffler: &dyn DeckShuffler,
    out: &broadcast::Sender<RoomEvent>,
) -> Reply {
    let result = match command {
        Command::Join(member) => room.join(member),
        Command::Leave(player) => room.leave(player, now),
        Command::Start(player) => room.start(player, &mut With(shuffler), now),
        Command::Bid(player, moves) => room.bid(player, moves, now),
        Command::Move(player, robot, dir) => room.move_robot(player, robot, dir, now),
        Command::Concede(player) => room.concede(player, now),
        Command::Tick => Ok(room.tick(now)),
        Command::Members => return Reply::Members(room.members().to_vec()),
    };
    Reply::Events(result.map(|events| publish(out, events)))
}

/// Broadcasts events; with no subscribers they are simply dropped.
fn publish(out: &broadcast::Sender<RoomEvent>, events: Vec<RoomEvent>) {
    for event in events {
        let _ = out.send(event);
    }
}
