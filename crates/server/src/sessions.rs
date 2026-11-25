//! Cookie sessions: who is signed in, and short-lived sign-in state.

use tower_sessions::cookie::SameSite;
use tower_sessions::{Expiry, SessionManagerLayer, SessionStore};

/// Name of the session cookie over plain HTTP (local development).
pub const COOKIE: &str = "rr_session";
/// Over HTTPS the `__Host-` prefix makes browsers refuse the cookie unless
/// it is Secure, host-only and on `/`, so a sibling subdomain or an HTTP
/// origin can't plant or overwrite it.
pub const SECURE_COOKIE: &str = "__Host-rr_session";

/// How long a session survives without requests.
pub const IDLE_EXPIRY: time::Duration = time::Duration::hours(24);

/// The session layer with the project's cookie policy: `HttpOnly`,
/// `SameSite=Lax` (after launch, the Google callback is a top-level GET from
/// another site, which `Strict` would strip the cookie from), `Secure` when
/// served over HTTPS, and expiry after a day of inactivity.
pub fn layer<S: SessionStore + Clone>(store: S, https: bool) -> SessionManagerLayer<S> {
    SessionManagerLayer::new(store)
        .with_name(if https { SECURE_COOKIE } else { COOKIE })
        .with_path("/")
        .with_http_only(true)
        .with_same_site(SameSite::Lax)
        .with_secure(https)
        .with_expiry(Expiry::OnInactivity(IDLE_EXPIRY))
}

/// Opens the SQLite session store on the game database's pool and creates
/// its table if missing.
///
/// # Errors
/// If the table can't be created.
pub async fn sqlite_store(
    store: &storage::SqliteStore,
) -> Result<tower_sessions_sqlx_store::SqliteStore, sqlx_error::Error> {
    let sessions = tower_sessions_sqlx_store::SqliteStore::new(store.pool().clone());
    sessions.migrate().await?;
    Ok(sessions)
}

/// Re-export so callers needn't depend on sqlx directly.
pub mod sqlx_error {
    pub use tower_sessions_sqlx_store::sqlx::Error;
}
