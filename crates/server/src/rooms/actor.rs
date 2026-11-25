//! One task per room owns its `RoomState`: commands come in over a channel,
//! each is answered, and the resulting events are broadcast. Nothing about
//! a room is shared or locked.

use std::sync::Arc;
use std::time::Duration;

use domain::{Board, Direction, PlayerId, RobotColor, Robots, Shuffle, Target, Timestamp};
use storage::GameResultRepository;
use tokio::sync::{broadcast, mpsc, oneshot};

use super::state::{Member, RoomError, RoomEvent, RoomState, RoomView};
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
    Chat(PlayerId, String),
    Tick,
    Members,
    View,
    CloseIfEmpty,
}

enum Reply {
    Events(Result<(), RoomError>),
    Members(Vec<Member>),
    View(Box<RoomView>),
    Closed(bool),
}

/// Why a call on a room failed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CallError {
    Room(RoomError),
    /// The room's task has stopped: it was closed, or every handle was
    /// dropped.
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
    /// and stops when closed while empty or once every handle is dropped.
    /// A finished game is recorded in `results` once, as it ends.
    #[must_use]
    pub fn spawn(
        board: Board,
        robots: Robots,
        capacity: usize,
        clock: Arc<dyn Clock>,
        shuffler: Arc<dyn DeckShuffler>,
        results: Arc<dyn GameResultRepository>,
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
                        if matches!(command, Command::CloseIfEmpty) {
                            let close = room.members().is_empty();
                            let _ = reply.send(Reply::Closed(close));
                            if close {
                                break;
                            }
                            continue;
                        }
                        let now = clock.now();
                        let answer = apply(&mut room, command, now, shuffler.as_ref());
                        let answer = match answer {
                            Applied::Events(Ok(events)) => {
                                deliver(&room, events, now, &out, &results);
                                Reply::Events(Ok(()))
                            }
                            Applied::Events(Err(e)) => Reply::Events(Err(e)),
                            Applied::Reply(reply) => reply,
                        };
                        let _ = reply.send(answer);
                    }
                    _ = ticker.tick() => {
                        let now = clock.now();
                        let events = room.tick(now);
                        deliver(&room, events, now, &out, &results);
                    }
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
            Reply::Members(_) | Reply::View(_) | Reply::Closed(_) => {
                unreachable!("commands answer with events")
            }
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
            Reply::Events(_) | Reply::View(_) | Reply::Closed(_) => {
                unreachable!("Members answers with members")
            }
        }
    }

    /// Stops the room if nobody is in it; every later call on any handle
    /// gets `Closed`. Returns whether it stopped. Deciding inside the task
    /// means a join that races the close either lands first (and keeps the
    /// room open) or gets `Closed`.
    ///
    /// # Errors
    /// `Closed` if it had already stopped.
    pub async fn close_if_empty(&self) -> Result<bool, CallError> {
        match self.call(Command::CloseIfEmpty).await? {
            Reply::Closed(closed) => Ok(closed),
            Reply::Events(_) | Reply::Members(_) | Reply::View(_) => {
                unreachable!("close answers with Closed")
            }
        }
    }

    /// The whole room as it is now, for a member who just (re)connected.
    /// Subscribe before asking, so no event falls between the two.
    ///
    /// # Errors
    /// `Closed`.
    pub async fn view(&self) -> Result<RoomView, CallError> {
        match self.call(Command::View).await? {
            Reply::View(view) => Ok(*view),
            Reply::Events(_) | Reply::Members(_) | Reply::Closed(_) => {
                unreachable!("View answers with the view")
            }
        }
    }

    /// # Errors
    /// The room's refusal, or `Closed`.
    pub async fn chat(&self, player: PlayerId, text: String) -> Result<(), CallError> {
        self.act(Command::Chat(player, text)).await
    }
}

/// What a command produced: events to deliver, or a direct answer.
enum Applied {
    Events(Result<Vec<RoomEvent>, RoomError>),
    Reply(Reply),
}

fn apply(
    room: &mut RoomState,
    command: Command,
    now: Timestamp,
    shuffler: &dyn DeckShuffler,
) -> Applied {
    Applied::Events(match command {
        Command::Join(member) => room.join(member),
        Command::Leave(player) => room.leave(player, now),
        Command::Start(player) => room.start(player, &mut With(shuffler), now),
        Command::Bid(player, moves) => room.bid(player, moves, now),
        Command::Move(player, robot, dir) => room.move_robot(player, robot, dir, now),
        Command::Concede(player) => room.concede(player, now),
        Command::Chat(player, text) => room.chat(player, &text),
        Command::Tick => Ok(room.tick(now)),
        Command::Members => return Applied::Reply(Reply::Members(room.members().to_vec())),
        Command::View => return Applied::Reply(Reply::View(Box::new(room.view()))),
        Command::CloseIfEmpty => unreachable!("handled by the task loop"),
    })
}

/// Broadcasts events (dropped if nobody listens) and, when the game has
/// just ended, records its result. `GameOver` is emitted once per game, so
/// the result is recorded once; storing it runs apart from the room so a
/// slow database never holds up play.
fn deliver(
    room: &RoomState,
    events: Vec<RoomEvent>,
    now: Timestamp,
    out: &broadcast::Sender<RoomEvent>,
    results: &Arc<dyn GameResultRepository>,
) {
    let over = events
        .iter()
        .any(|e| matches!(e, RoomEvent::GameOver { .. }));
    if over && let Some(game) = room.result(now) {
        let results = results.clone();
        tokio::spawn(async move {
            if let Err(e) = results.record(&game).await {
                tracing::error!(error = %e, "recording a finished game failed");
            }
        });
    }
    for event in events {
        let _ = out.send(event);
    }
}
