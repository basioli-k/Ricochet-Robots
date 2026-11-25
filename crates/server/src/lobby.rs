//! The lobby's HTTP side: opening rooms and listing them.

use std::net::SocketAddr;

use axum::Json;
use axum::extract::{ConnectInfo, State};
use axum::http::{HeaderMap, StatusCode};
use serde::Serialize;
use tower_sessions::Session;
use ts_rs::TS;

use crate::AppState;
use crate::identity::{self, ApiError};
use crate::rate_limit::client_key;
use crate::rooms::manager::{CreateError, Creator, RoomSummary};

/// The new room's ID, answered to `POST /api/rooms`.
#[derive(Debug, Serialize, TS)]
#[ts(export, rename = "RoomCreated")]
pub struct Created {
    pub id: String,
}

const fn create_error(e: CreateError) -> ApiError {
    match e {
        CreateError::TooManyRooms => ApiError {
            status: StatusCode::SERVICE_UNAVAILABLE,
            code: "too_many_rooms",
        },
        CreateError::OwnerLimit => ApiError {
            status: StatusCode::CONFLICT,
            code: "room_limit",
        },
        CreateError::RateLimited => ApiError {
            status: StatusCode::TOO_MANY_REQUESTS,
            code: "rate_limited",
        },
    }
}

/// `POST /api/rooms`: opens a room for the signed-in player, counted
/// against them and their client (IPv4 address or IPv6 /64).
///
/// # Errors
/// `csrf_mismatch`, `not_signed_in`, `too_many_rooms`, `room_limit` or
/// `rate_limited`.
pub async fn create(
    State(state): State<AppState>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    session: Session,
    headers: HeaderMap,
) -> Result<(StatusCode, Json<Created>), ApiError> {
    identity::require_csrf(&session, &headers).await?;
    let player = identity::current(&session)
        .await?
        .ok_or(ApiError::NOT_SIGNED_IN)?;
    let client = client_key(peer.ip());
    let (id, _) = state
        .rooms
        .create(Creator {
            player: player.id(),
            client: &client,
        })
        .map_err(create_error)?;
    Ok((StatusCode::CREATED, Json(Created { id })))
}

/// `GET /api/rooms`: open rooms as `[{id, members}]`, sorted by ID.
pub async fn list(State(state): State<AppState>) -> Json<Vec<RoomSummary>> {
    Json(state.rooms.list().await)
}
