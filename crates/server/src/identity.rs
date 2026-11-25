//! Who the session belongs to, and the CSRF token guarding its changes.

use axum::Json;
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use serde::{Deserialize, Serialize};
use tower_sessions::Session;

const IDENTITY: &str = "identity";
const CSRF: &str = "csrf";

/// Header carrying the session's CSRF token on state-changing requests.
pub const CSRF_HEADER: &str = "x-csrf-token";

/// The signed-in player. Only guests exist until Google sign-in ships.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Identity {
    Guest {
        /// The in-game `PlayerId`, random so it never collides with others.
        id: String,
        nickname: String,
    },
}

/// A failure the client shows by its stable `code` (translated client-side).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ApiError {
    pub status: StatusCode,
    pub code: &'static str,
}

impl ApiError {
    pub const CSRF: Self = Self {
        status: StatusCode::FORBIDDEN,
        code: "csrf_mismatch",
    };
    pub const INTERNAL: Self = Self {
        status: StatusCode::INTERNAL_SERVER_ERROR,
        code: "internal",
    };
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (self.status, Json(serde_json::json!({ "error": self.code }))).into_response()
    }
}

fn internal(e: impl std::fmt::Display) -> ApiError {
    tracing::error!(error = %e, "session store failure");
    ApiError::INTERNAL
}

/// 128 random bits as hex, from the thread's cryptographically secure RNG.
#[must_use]
pub fn random_token() -> String {
    format!("{:032x}", rand::random::<u128>())
}

/// The session's CSRF token, created on first use.
///
/// # Errors
/// If the session store fails.
pub async fn csrf_token(session: &Session) -> Result<String, ApiError> {
    if let Some(token) = session.get::<String>(CSRF).await.map_err(internal)? {
        return Ok(token);
    }
    let token = random_token();
    session.insert(CSRF, &token).await.map_err(internal)?;
    Ok(token)
}

/// Rejects the request unless it echoes the session's CSRF token. The
/// comparison doesn't stop at the first differing byte.
///
/// # Errors
/// [`ApiError::CSRF`] on a missing or wrong token.
pub async fn require_csrf(session: &Session, headers: &HeaderMap) -> Result<(), ApiError> {
    let expected = session.get::<String>(CSRF).await.map_err(internal)?;
    let sent = headers.get(CSRF_HEADER).and_then(|v| v.to_str().ok());
    match (expected, sent) {
        (Some(expected), Some(sent)) if constant_time_eq(expected.as_bytes(), sent.as_bytes()) => {
            Ok(())
        }
        _ => Err(ApiError::CSRF),
    }
}

fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    a.len() == b.len() && a.iter().zip(b).fold(0u8, |acc, (x, y)| acc | (x ^ y)) == 0
}

/// # Errors
/// If the session store fails.
pub async fn current(session: &Session) -> Result<Option<Identity>, ApiError> {
    session.get(IDENTITY).await.map_err(internal)
}

/// Signs `identity` in on a fresh session ID (so an ID planted before
/// sign-in is worthless) with a fresh CSRF token, which it returns.
///
/// # Errors
/// If the session store fails.
pub async fn sign_in(session: &Session, identity: &Identity) -> Result<String, ApiError> {
    session.cycle_id().await.map_err(internal)?;
    session.insert(IDENTITY, identity).await.map_err(internal)?;
    let token = random_token();
    session.insert(CSRF, &token).await.map_err(internal)?;
    Ok(token)
}

#[derive(Debug, Serialize)]
pub struct SessionView {
    pub csrf: String,
    pub identity: Option<Identity>,
}

/// `GET /api/session`: who is signed in, and the CSRF token to send back.
///
/// # Errors
/// If the session store fails.
pub async fn get_session(session: Session) -> Result<Json<SessionView>, ApiError> {
    Ok(Json(SessionView {
        csrf: csrf_token(&session).await?,
        identity: current(&session).await?,
    }))
}

#[cfg(test)]
mod tests {
    use super::constant_time_eq;

    #[test]
    fn compares_equal_and_unequal_tokens() {
        assert!(constant_time_eq(b"abc", b"abc"));
        assert!(!constant_time_eq(b"abc", b"abd"));
        assert!(!constant_time_eq(b"abc", b"ab"));
    }
}
