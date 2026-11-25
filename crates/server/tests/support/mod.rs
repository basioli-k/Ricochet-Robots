//! Helpers shared by the HTTP test crates. Each crate uses a subset, so
//! unused ones are allowed here rather than flagged per crate.
#![allow(dead_code)]

use std::net::SocketAddr;
use std::sync::Arc;

use axum::Router;
use axum::body::Body;
use axum::extract::connect_info::MockConnectInfo;
use axum::http::{Request, StatusCode, header};
use domain::{PlayerId, Timestamp, fixed_board};
use http_body_util::BodyExt;
use serde_json::{Value, json};
use server::clock::FakeClock;
use server::identity::CSRF_HEADER;
use server::rate_limit::SlidingWindow;
use server::rooms::actor::RandomShuffler;
use server::rooms::manager::{RoomLimits, RoomManager};
use server::{AppState, app, sessions};
use storage::{Account, InMemoryStore, Locale, PlayerRepository};
use tower::ServiceExt;
use tower_sessions::MemoryStore;

pub const CLIENT: [u8; 4] = [203, 0, 113, 7];

/// Limits for a test app; every window is one minute.
#[derive(Debug, Clone, Copy)]
pub struct Limits {
    pub guest_starts: usize,
    pub rooms: RoomLimits,
    pub room_creates: usize,
}

impl Default for Limits {
    fn default() -> Self {
        Self {
            guest_starts: 100,
            rooms: RoomLimits::default(),
            room_creates: 100,
        }
    }
}

/// One app per client address, sharing state and sessions, with an
/// account named "ana".
pub async fn apps_at(limits: Limits, clients: &[SocketAddr]) -> (Vec<Router>, Arc<FakeClock>) {
    let store = Arc::new(InMemoryStore::new());
    let ana = Account {
        id: PlayerId::new(1),
        google_subject: "google-1".into(),
        username: "ana".into(),
        locale: Locale::En,
        created_at: Timestamp(0),
    };
    store.create(&ana).await.unwrap();
    let clock = Arc::new(FakeClock::at(1_000));
    let rooms = RoomManager::new(
        limits.rooms,
        || (fixed_board::board(), fixed_board::starting_robots()),
        clock.clone(),
        Arc::new(RandomShuffler),
        Arc::new(SlidingWindow::new(limits.room_creates, 60_000)),
    );
    let state = AppState {
        players: store.clone(),
        games: store,
        clock: clock.clone(),
        guest_limiter: Arc::new(SlidingWindow::new(limits.guest_starts, 60_000)),
        rooms: Arc::new(rooms),
    };
    let sessions = MemoryStore::default();
    let apps = clients
        .iter()
        .map(|&client| {
            app(state.clone(), sessions::layer(sessions.clone(), false))
                .layer(MockConnectInfo(client))
        })
        .collect();
    (apps, clock)
}

/// An app whose clients all connect from one IPv4 address.
pub async fn app_with(limits: Limits) -> Router {
    let (mut apps, _) = apps_at(limits, &[SocketAddr::from((CLIENT, 4000))]).await;
    apps.remove(0)
}

/// The cookie pair from a response's Set-Cookie, if any.
pub fn cookie(response: &axum::response::Response) -> Option<String> {
    let value = response.headers().get(header::SET_COOKIE)?.to_str().ok()?;
    Some(value.split(';').next()?.to_owned())
}

pub async fn json_body(response: axum::response::Response) -> Value {
    serde_json::from_slice(&response.into_body().collect().await.unwrap().to_bytes()).unwrap()
}

/// GET /api/session: returns (cookie, csrf, identity).
pub async fn session(app: &Router, cookie_pair: Option<&str>) -> (Option<String>, String, Value) {
    let mut request = Request::get("/api/session");
    if let Some(pair) = cookie_pair {
        request = request.header(header::COOKIE, pair);
    }
    let response = app
        .clone()
        .oneshot(request.body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let set = cookie(&response);
    let body = json_body(response).await;
    (
        set,
        body["csrf"].as_str().unwrap().to_owned(),
        body["identity"].clone(),
    )
}

/// POST /api/guest.
pub async fn start(
    app: &Router,
    cookie_pair: &str,
    csrf: &str,
    nickname: &str,
) -> axum::response::Response {
    let request = Request::post("/api/guest")
        .header(header::COOKIE, cookie_pair)
        .header(CSRF_HEADER, csrf)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(json!({ "nickname": nickname }).to_string()))
        .unwrap();
    app.clone().oneshot(request).await.unwrap()
}

/// Signs a guest in; returns their (cookie, csrf, player id).
pub async fn guest(app: &Router, nickname: &str) -> (String, String, String) {
    let (pair, csrf, _) = session(app, None).await;
    let response = start(app, &pair.unwrap(), &csrf, nickname).await;
    assert_eq!(response.status(), StatusCode::OK);
    let pair = cookie(&response).unwrap();
    let body = json_body(response).await;
    (
        pair,
        body["csrf"].as_str().unwrap().to_owned(),
        body["identity"]["id"].as_str().unwrap().to_owned(),
    )
}
