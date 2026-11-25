//! The game socket, `GET /api/rooms/{id}/ws`. Opening it joins the room as
//! the session's player and sends a snapshot; after that every room event
//! is forwarded and every client message becomes a room command for that
//! player. Closing it, or going quiet past the heartbeat timeout, leaves the
//! room. A player with two sockets in one room leaves when either closes.

use std::time::Duration;

use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode, header};
use axum::response::Response;
use tokio::sync::broadcast::error::RecvError;
use tokio::time::Instant;
use tower_sessions::Session;

use crate::AppState;
use crate::identity::{self, ApiError, Identity};
use crate::protocol::{ClientMessage, ServerMessage, parse_player_id, player_id, room_error_code};
use crate::rooms::actor::{CallError, RoomHandle};
use crate::rooms::state::Member;

/// Largest message a client may send; real ones are well under 1 KiB.
const MAX_MESSAGE_BYTES: usize = 4 * 1024;

/// How often the server pings, and how long a silent socket is kept.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Heartbeat {
    pub every: Duration,
    pub timeout: Duration,
}

impl Default for Heartbeat {
    fn default() -> Self {
        Self {
            every: Duration::from_secs(20),
            timeout: Duration::from_mins(1),
        }
    }
}

const BAD_ORIGIN: ApiError = ApiError {
    status: StatusCode::FORBIDDEN,
    code: "bad_origin",
};
const ROOM_NOT_FOUND: ApiError = ApiError {
    status: StatusCode::NOT_FOUND,
    code: "room_not_found",
};

/// Upgrades to the game socket. Browsers attach cookies to WebSocket
/// requests from any site, so the `Origin` must be our own.
///
/// # Errors
/// `bad_origin`, `not_signed_in` or `room_not_found`.
pub async fn connect(
    State(state): State<AppState>,
    Path(room_id): Path<String>,
    session: Session,
    headers: HeaderMap,
    upgrade: WebSocketUpgrade,
) -> Result<Response, ApiError> {
    let origin = headers.get(header::ORIGIN).and_then(|v| v.to_str().ok());
    if origin != Some(state.public_origin.as_str()) {
        return Err(BAD_ORIGIN);
    }
    let identity = identity::current(&session)
        .await?
        .ok_or(ApiError::NOT_SIGNED_IN)?;
    let me = member(&identity).ok_or(ApiError::INTERNAL)?;
    let room = state.rooms.get(&room_id).ok_or(ROOM_NOT_FOUND)?;
    Ok(upgrade
        .max_message_size(MAX_MESSAGE_BYTES)
        .max_frame_size(MAX_MESSAGE_BYTES)
        .on_upgrade(move |socket| play(socket, room, me, state)))
}

fn member(identity: &Identity) -> Option<Member> {
    match identity {
        Identity::Guest { id, nickname } => Some(Member {
            id: parse_player_id(id)?,
            name: nickname.clone(),
            is_guest: true,
        }),
    }
}

async fn send(socket: &mut WebSocket, message: &ServerMessage) -> Result<(), ()> {
    let text = serde_json::to_string(message).map_err(|_| ())?;
    socket
        .send(Message::Text(text.into()))
        .await
        .map_err(|_| ())
}

async fn snapshot(socket: &mut WebSocket, room: &RoomHandle, me: &Member) -> Result<(), ()> {
    let view = room.view().await.map_err(|_| ())?;
    let message = ServerMessage::Snapshot {
        you: player_id(me.id),
        room: (&view).into(),
    };
    send(socket, &message).await
}

const fn call_code(e: CallError) -> &'static str {
    match e {
        CallError::Room(e) => room_error_code(e),
        CallError::Closed => "room_closed",
    }
}

async fn play(mut socket: WebSocket, room: RoomHandle, me: Member, state: AppState) {
    if let Err(e) = room.join(me.clone()).await {
        let _ = send(&mut socket, &ServerMessage::error(call_code(e))).await;
        return;
    }
    // Subscribe before taking the snapshot, so no event falls between the
    // two (one may show in both); earlier ones, like our own join, are
    // already in the snapshot.
    let mut events = room.subscribe();
    if snapshot(&mut socket, &room, &me).await.is_ok() {
        converse(&mut socket, &room, &me, &state, &mut events).await;
    }
    let _ = room.leave(me.id).await;
}

async fn converse(
    socket: &mut WebSocket,
    room: &RoomHandle,
    me: &Member,
    state: &AppState,
    events: &mut tokio::sync::broadcast::Receiver<crate::rooms::state::RoomEvent>,
) {
    let mut heartbeat = tokio::time::interval(state.heartbeat.every);
    heartbeat.tick().await;
    let mut last_heard = Instant::now();
    loop {
        tokio::select! {
            incoming = socket.recv() => {
                let Some(Ok(message)) = incoming else { return };
                last_heard = Instant::now();
                let reply = match message {
                    Message::Text(text) => match handle(state, room, me, &text).await {
                        Some(Outcome::Leave) => return,
                        Some(Outcome::Refused(code)) => Some(code),
                        None => None,
                    },
                    Message::Binary(_) => Some("bad_message"),
                    Message::Close(_) => return,
                    // Pings are answered by the library; any frame proves life.
                    Message::Ping(_) | Message::Pong(_) => None,
                };
                if let Some(code) = reply
                    && send(socket, &ServerMessage::error(code)).await.is_err()
                {
                    return;
                }
            }
            event = events.recv() => {
                let sent = match event {
                    Ok(event) => send(socket, &event.into()).await,
                    // Too slow to keep up: start again from a fresh snapshot.
                    Err(RecvError::Lagged(_)) => snapshot(socket, room, me).await,
                    Err(RecvError::Closed) => Err(()),
                };
                if sent.is_err() {
                    return;
                }
            }
            _ = heartbeat.tick() => {
                if last_heard.elapsed() >= state.heartbeat.timeout
                    || socket.send(Message::Ping(Vec::new().into())).await.is_err()
                {
                    return;
                }
            }
        }
    }
}

enum Outcome {
    Leave,
    Refused(&'static str),
}

/// Runs one client message as the socket's player; `None` when it worked.
async fn handle(state: &AppState, room: &RoomHandle, me: &Member, text: &str) -> Option<Outcome> {
    let Ok(message) = serde_json::from_str::<ClientMessage>(text) else {
        return Some(Outcome::Refused("bad_message"));
    };
    let now = state.clock.now();
    let key = player_id(me.id);
    let allowed = match &message {
        ClientMessage::Chat { .. } => state.chat_limiter.allow(&key, now),
        ClientMessage::Bid { .. } | ClientMessage::Move { .. } => {
            state.action_limiter.allow(&key, now)
        }
        ClientMessage::Start | ClientMessage::Concede | ClientMessage::Leave => true,
    };
    if !allowed {
        return Some(Outcome::Refused("rate_limited"));
    }
    let player = me.id;
    let result = match message {
        ClientMessage::Start => room.start(player).await,
        ClientMessage::Bid { moves } => room.bid(player, moves).await,
        ClientMessage::Move { robot, dir } => {
            room.move_robot(player, robot.into(), dir.into()).await
        }
        ClientMessage::Concede => room.concede(player).await,
        ClientMessage::Chat { text } => room.chat(player, text).await,
        ClientMessage::Leave => return Some(Outcome::Leave),
    };
    result.err().map(|e| Outcome::Refused(call_code(e)))
}
