//! The session cookie policy, and sessions surviving a restart in SQLite.

use std::path::PathBuf;

use axum::body::Body;
use axum::http::{Request, header};
use axum::{Router, routing::get};
use http_body_util::BodyExt;
use server::sessions;
use tower::ServiceExt;
use tower_sessions::{MemoryStore, Session, SessionManagerLayer, SessionStore};

/// `/write` stores a value in the session; `/read` returns it.
fn probe<S: SessionStore + Clone>(layer: SessionManagerLayer<S>) -> Router {
    Router::new()
        .route(
            "/write",
            get(|session: Session| async move {
                session.insert("probe", "kept").await.unwrap();
            }),
        )
        .route(
            "/read",
            get(|session: Session| async move {
                session
                    .get::<String>("probe")
                    .await
                    .unwrap()
                    .unwrap_or_default()
            }),
        )
        .layer(layer)
}

async fn set_cookie(app: Router) -> String {
    let response = app
        .oneshot(Request::get("/write").body(Body::empty()).unwrap())
        .await
        .unwrap();
    response.headers()[header::SET_COOKIE]
        .to_str()
        .unwrap()
        .to_owned()
}

#[tokio::test]
async fn the_cookie_is_http_only_lax_and_expires() {
    let cookie = set_cookie(probe(sessions::layer(MemoryStore::default(), false))).await;
    assert!(
        cookie.starts_with(&format!("{}=", sessions::COOKIE)),
        "{cookie}"
    );
    for attribute in ["HttpOnly", "SameSite=Lax", "Max-Age=86400"] {
        assert!(cookie.contains(attribute), "missing {attribute}: {cookie}");
    }
    assert!(!cookie.contains("Secure"), "{cookie}");
}

#[tokio::test]
async fn the_cookie_is_secure_over_https() {
    let cookie = set_cookie(probe(sessions::layer(MemoryStore::default(), true))).await;
    assert!(
        cookie.starts_with(&format!("{}=", sessions::SECURE_COOKIE)),
        "{cookie}"
    );
    for attribute in ["Secure", "Path=/"] {
        assert!(cookie.contains(attribute), "missing {attribute}: {cookie}");
    }
    assert!(!cookie.contains("Domain"), "{cookie}");
}

/// A fresh directory removed on drop, even if the test panics.
struct TempDir(PathBuf);

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

#[tokio::test]
async fn sessions_survive_a_restart_in_sqlite() {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_nanos());
    let dir =
        TempDir(std::env::temp_dir().join(format!("rr-sessions-{}-{nanos}", std::process::id())));
    std::fs::create_dir(&dir.0).unwrap();
    let url = format!("sqlite://{}", dir.0.join("db.sqlite").display());

    let cookie = {
        let store = storage::SqliteStore::connect(&url).await.unwrap();
        let app = probe(sessions::layer(
            sessions::sqlite_store(&store).await.unwrap(),
            false,
        ));
        set_cookie(app).await
    };
    // A new store and app on the same file stand in for a restarted server.
    let store = storage::SqliteStore::connect(&url).await.unwrap();
    let app = probe(sessions::layer(
        sessions::sqlite_store(&store).await.unwrap(),
        false,
    ));
    let pair = cookie.split(';').next().unwrap().to_owned();
    let response = app
        .oneshot(
            Request::get("/read")
                .header(header::COOKIE, pair)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let body = response.into_body().collect().await.unwrap().to_bytes();
    assert_eq!(&body[..], b"kept");
}
