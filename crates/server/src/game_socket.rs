//! The game socket, `GET /api/rooms/{id}/ws`. Opening it joins the room as
//! the session's player and sends a snapshot; after that every room event
//! is forwarded and every client message becomes a room command for that
//! player. Closing it, going quiet past the heartbeat timeout, not reading
//! fast enough (a send waits at most that long) or falling behind the
//! room's events leaves the room. A player with two sockets in one room
//! leaves when either closes.

use std::time::Duration;

use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::{ConnectInfo, Path, State};
use axum::http::{HeaderMap, StatusCode, header};
use axum::response::Response;
use tokio::sync::broadcast::error::RecvError;
use tokio::time::Instant;
use tower_sessions::Session;

use crate::AppState;
use crate::connections::Ticket;
use crate::identity::{self, ApiError, Identity};
use crate::protocol::{ClientMessage, ServerMessage, parse_player_id, player_id, room_error_code};
use crate::rate_limit::client_key;
use crate::rooms::actor::{CallError, RoomHandle};
use crate::rooms::state::{Member, RoomEvent};

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
const TOO_MANY_CONNECTIONS: ApiError = ApiError {
    status: StatusCode::TOO_MANY_REQUESTS,
    code: "too_many_connections",
};

/// Upgrades to the game socket. Browsers attach cookies to WebSocket
/// requests from any site, so the `Origin` must be our own.
///
/// # Errors
/// `bad_origin`, `not_signed_in`, `room_not_found` or
/// `too_many_connections` (per player, and per client address).
pub async fn connect(
    State(state): State<AppState>,
    ConnectInfo(peer): ConnectInfo<std::net::SocketAddr>,
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
    let ticket = state
        .connections
        .open(identity.id(), &client_key(peer.ip()))
        .ok_or(TOO_MANY_CONNECTIONS)?;
    Ok(upgrade
        .max_message_size(MAX_MESSAGE_BYTES)
        .max_frame_size(MAX_MESSAGE_BYTES)
        .on_upgrade(move |socket| {
            let link = Link {
                socket,
                patience: state.heartbeat.timeout,
                _ticket: ticket,
            };
            play(link, room, me, state)
        }))
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

/// The socket, with a deadline on every send: a client that stops reading
/// fills its buffers, and waiting on it forever would keep the player in
/// the room and its task alive.
struct Link {
    socket: WebSocket,
    patience: Duration,
    /// Holds this socket's place under the connection caps.
    _ticket: Ticket,
}

impl Link {
    async fn send(&mut self, message: Message) -> Result<(), ()> {
        match tokio::time::timeout(self.patience, self.socket.send(message)).await {
            Ok(Ok(())) => Ok(()),
            _ => Err(()),
        }
    }

    async fn send_json(&mut self, message: &ServerMessage) -> Result<(), ()> {
        let text = serde_json::to_string(message).map_err(|_| ())?;
        self.send(Message::Text(text.into())).await
    }

    async fn snapshot(&mut self, room: &RoomHandle, me: &Member) -> Result<(), ()> {
        let view = room.view().await.map_err(|_| ())?;
        let message = ServerMessage::Snapshot {
            you: player_id(me.id),
            room: (&view).into(),
        };
        self.send_json(&message).await
    }
}

const fn call_code(e: CallError) -> &'static str {
    match e {
        CallError::Room(e) => room_error_code(e),
        CallError::Closed => "room_closed",
    }
}

/// What to do with the next item from the room's event stream.
#[derive(Debug, PartialEq, Eq)]
enum Forward {
    Send(Box<ServerMessage>),
    /// Tell the client why, then close. A client that fell behind (`lagged`)
    /// reconnects and starts again from a fresh snapshot.
    Close(&'static str),
}

fn forward(event: Result<RoomEvent, RecvError>) -> Forward {
    match event {
        Ok(event) => Forward::Send(Box::new(event.into())),
        Err(RecvError::Lagged(_)) => Forward::Close("lagged"),
        Err(RecvError::Closed) => Forward::Close("room_closed"),
    }
}

async fn play(mut link: Link, room: RoomHandle, me: Member, state: AppState) {
    if let Err(e) = room.join(me.clone()).await {
        let _ = link.send_json(&ServerMessage::error(call_code(e))).await;
        return;
    }
    // Subscribe before taking the snapshot, so no event falls between the
    // two (one may show in both); earlier ones, like our own join, are
    // already in the snapshot.
    let mut events = room.subscribe();
    if link.snapshot(&room, &me).await.is_ok() {
        converse(&mut link, &room, &me, &state, &mut events).await;
    }
    let _ = room.leave(me.id).await;
}

async fn converse(
    link: &mut Link,
    room: &RoomHandle,
    me: &Member,
    state: &AppState,
    events: &mut tokio::sync::broadcast::Receiver<RoomEvent>,
) {
    let mut heartbeat = tokio::time::interval(state.heartbeat.every);
    heartbeat.tick().await;
    let mut last_heard = Instant::now();
    loop {
        tokio::select! {
            incoming = link.socket.recv() => {
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
                    && link.send_json(&ServerMessage::error(code)).await.is_err()
                {
                    return;
                }
            }
            event = events.recv() => match forward(event) {
                Forward::Send(message) => {
                    if link.send_json(&message).await.is_err() {
                        return;
                    }
                }
                Forward::Close(code) => {
                    let _ = link.send_json(&ServerMessage::error(code)).await;
                    return;
                }
            },
            _ = heartbeat.tick() => {
                if last_heard.elapsed() >= state.heartbeat.timeout
                    || link.send(Message::Ping(Vec::new().into())).await.is_err()
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

#[cfg(test)]
mod tests {
    use domain::PlayerId;

    use super::*;

    #[test]
    fn events_are_forwarded_and_a_lagging_socket_is_closed() {
        let left = RoomEvent::Left(PlayerId::new(1));
        assert_eq!(
            forward(Ok(left.clone())),
            Forward::Send(Box::new(left.into()))
        );
        assert_eq!(forward(Err(RecvError::Lagged(3))), Forward::Close("lagged"));
        assert_eq!(
            forward(Err(RecvError::Closed)),
            Forward::Close("room_closed")
        );
    }
}
