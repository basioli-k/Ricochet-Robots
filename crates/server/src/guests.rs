//! Starting a guest session: no account, just a nickname.

use std::net::SocketAddr;

use axum::Json;
use axum::extract::{ConnectInfo, State};
use axum::http::{HeaderMap, StatusCode};
use serde::{Deserialize, Serialize};
use tower_sessions::Session;

use crate::AppState;
use crate::identity::{self, ApiError, Identity};
use crate::names::PlayerName;

/// Guest starts allowed per client IP within [`GUEST_WINDOW_MS`].
pub const GUEST_STARTS: usize = 5;
pub const GUEST_WINDOW_MS: u64 = 10 * 60 * 1000;

const INVALID: ApiError = ApiError {
    status: StatusCode::BAD_REQUEST,
    code: "nickname_invalid",
};
const TAKEN: ApiError = ApiError {
    status: StatusCode::CONFLICT,
    code: "nickname_taken",
};
const LIMITED: ApiError = ApiError {
    status: StatusCode::TOO_MANY_REQUESTS,
    code: "rate_limited",
};

#[derive(Debug, Deserialize)]
pub struct StartGuest {
    pub nickname: String,
}

#[derive(Debug, Serialize)]
pub struct Started {
    pub identity: Identity,
    /// The session's new CSRF token; the old one no longer works.
    pub csrf: String,
}

/// `POST /api/guest`. Checks run cheapest-first: CSRF, rate limit, name
/// rule, then whether an account already has the name (ignoring case), so
/// a guest can't pose as that account.
///
/// # Errors
/// `csrf_mismatch`, `rate_limited`, `nickname_invalid` or `nickname_taken`.
pub async fn start(
    State(state): State<AppState>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    session: Session,
    headers: HeaderMap,
    Json(body): Json<StartGuest>,
) -> Result<Json<Started>, ApiError> {
    identity::require_csrf(&session, &headers).await?;
    if !state
        .guest_limiter
        .allow(&peer.ip().to_string(), state.clock.now())
    {
        return Err(LIMITED);
    }
    let name = PlayerName::parse(&body.nickname).ok_or(INVALID)?;
    let taken = state
        .players
        .by_username(name.as_str())
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "account lookup failed");
            ApiError::INTERNAL
        })?;
    if taken.is_some() {
        return Err(TAKEN);
    }
    let identity = Identity::Guest {
        id: identity::random_token(),
        nickname: name.as_str().to_owned(),
    };
    let csrf = identity::sign_in(&session, &identity).await?;
    Ok(Json(Started { identity, csrf }))
}
