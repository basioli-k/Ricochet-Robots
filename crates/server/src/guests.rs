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
use crate::rate_limit::client_key;

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

/// `POST /api/guest`. Checks run cheapest-first: CSRF, the per-IP
/// `guest_limiter` (set by `RR_GUEST_STARTS` and `RR_GUEST_WINDOW_SECS`),
/// the name rule, then whether an account already has the name (ignoring
/// case), so a guest can't pose as that account.
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
        .allow(&client_key(peer.ip()), state.clock.now())
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
