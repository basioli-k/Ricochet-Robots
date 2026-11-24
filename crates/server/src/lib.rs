//! HTTP and WebSocket server: authentication, rooms and the game protocol.

pub mod clock;
pub mod config;

use std::sync::Arc;

use axum::{Router, routing::get};
use storage::{GameResultRepository, PlayerRepository};

use crate::clock::Clock;

/// Everything handlers depend on, injected so tests can swap in fakes.
#[derive(Clone)]
pub struct AppState {
    pub players: Arc<dyn PlayerRepository>,
    pub games: Arc<dyn GameResultRepository>,
    pub clock: Arc<dyn Clock>,
}

impl std::fmt::Debug for AppState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AppState").finish_non_exhaustive()
    }
}

/// The application's routes.
pub fn app(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .with_state(state)
}

/// For load balancers and uptime checks; not user-facing.
async fn health() -> &'static str {
    "ok"
}
