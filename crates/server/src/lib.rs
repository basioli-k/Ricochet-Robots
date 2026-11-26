//! HTTP and WebSocket server: authentication, rooms and the game protocol.

pub mod clock;
pub mod config;
pub mod connections;
pub mod game_socket;
pub mod guests;
pub mod identity;
pub mod lobby;
pub mod names;
pub mod protocol;
pub mod rate_limit;
pub mod rooms;
pub mod sessions;
pub mod web;

use std::sync::Arc;

use axum::{
    Router,
    routing::{get, post},
};
use storage::{GameResultRepository, PlayerRepository};
use tower_sessions::{SessionManagerLayer, SessionStore};

use crate::clock::Clock;
use crate::rate_limit::RateLimiter;
use crate::rooms::manager::RoomManager;

/// Everything handlers depend on, injected so tests can swap in fakes.
#[derive(Clone)]
pub struct AppState {
    pub players: Arc<dyn PlayerRepository>,
    pub games: Arc<dyn GameResultRepository>,
    pub clock: Arc<dyn Clock>,
    /// Limits guest starts per client IP.
    pub guest_limiter: Arc<dyn RateLimiter>,
    pub rooms: Arc<RoomManager>,
    /// Chat messages per player, and bids and moves per player.
    pub chat_limiter: Arc<dyn RateLimiter>,
    pub action_limiter: Arc<dyn RateLimiter>,
    /// The only `Origin` allowed to open a game socket.
    pub public_origin: String,
    pub heartbeat: game_socket::Heartbeat,
    /// Caps open game sockets per player and per client.
    pub connections: Arc<connections::Connections>,
}

impl std::fmt::Debug for AppState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AppState").finish_non_exhaustive()
    }
}

/// The application's routes, with sessions kept in `sessions` (SQLite in
/// production, in memory in tests).
pub fn app<S: SessionStore + Clone>(state: AppState, sessions: SessionManagerLayer<S>) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/api/session", get(identity::get_session))
        .route("/api/guest", post(guests::start))
        .route("/api/rooms", get(lobby::list).post(lobby::create))
        .route("/api/rooms/{id}/ws", get(game_socket::connect))
        .with_state(state)
        .layer(sessions)
}

/// For load balancers and uptime checks; not user-facing.
async fn health() -> &'static str {
    "ok"
}
