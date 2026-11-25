//! Starting a guest session through the HTTP API.

use std::net::SocketAddr;
use std::sync::Arc;

use axum::Router;
use axum::body::Body;
use axum::extract::connect_info::MockConnectInfo;
use axum::http::{Request, StatusCode, header};
use domain::{PlayerId, Timestamp};
use http_body_util::BodyExt;
use serde_json::{Value, json};
use server::clock::FakeClock;
use server::identity::CSRF_HEADER;
use server::rate_limit::SlidingWindow;
use server::{AppState, app, sessions};
use storage::{Account, InMemoryStore, Locale, PlayerRepository};
use tower::ServiceExt;
use tower_sessions::MemoryStore;

/// An app whose clients all connect from one address, with an account
/// named "ana" and a guest-start limit of `limit` per window.
async fn app_with(limit: usize) -> Router {
    let store = Arc::new(InMemoryStore::new());
    let ana = Account {
        id: PlayerId::new(1),
        google_subject: "google-1".into(),
        username: "ana".into(),
        locale: Locale::En,
        created_at: Timestamp(0),
    };
    store.create(&ana).await.unwrap();
    let state = AppState {
        players: store.clone(),
        games: store,
        clock: Arc::new(FakeClock::at(1_000)),
        guest_limiter: Arc::new(SlidingWindow::new(limit, 60_000)),
    };
    app(state, sessions::layer(MemoryStore::default(), false))
        .layer(MockConnectInfo(SocketAddr::from(([203, 0, 113, 7], 4000))))
}

/// The cookie pair from a response's Set-Cookie, if any.
fn cookie(response: &axum::response::Response) -> Option<String> {
    let value = response.headers().get(header::SET_COOKIE)?.to_str().ok()?;
    Some(value.split(';').next()?.to_owned())
}

async fn json_body(response: axum::response::Response) -> Value {
    serde_json::from_slice(&response.into_body().collect().await.unwrap().to_bytes()).unwrap()
}

/// GET /api/session: returns (cookie, csrf, identity).
async fn session(app: &Router, cookie_pair: Option<&str>) -> (Option<String>, String, Value) {
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

async fn start(
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

#[tokio::test]
async fn a_guest_starts_on_a_fresh_session() {
    let app = app_with(5).await;
    let (first, csrf, identity) = session(&app, None).await;
    let first = first.unwrap();
    assert_eq!(identity, Value::Null);

    let response = start(&app, &first, &csrf, "Visitor1").await;
    assert_eq!(response.status(), StatusCode::OK);
    let second = cookie(&response).expect("a new session cookie");
    assert_ne!(second, first, "the session ID is cycled");
    let body = json_body(response).await;
    assert_eq!(body["identity"]["kind"], "guest");
    assert_eq!(body["identity"]["nickname"], "Visitor1");
    assert_ne!(body["csrf"], csrf, "the CSRF token is renewed");

    let (_, _, identity) = session(&app, Some(&second)).await;
    assert_eq!(identity["nickname"], "Visitor1");
    let (_, _, before) = session(&app, Some(&first)).await;
    assert_eq!(
        before,
        Value::Null,
        "the pre-sign-in session ID is worthless"
    );
}

#[tokio::test]
async fn a_missing_or_wrong_csrf_token_is_refused() {
    let app = app_with(5).await;
    let (pair, _, _) = session(&app, None).await;
    let response = start(&app, &pair.unwrap(), "wrong", "Visitor1").await;
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
    assert_eq!(json_body(response).await["error"], "csrf_mismatch");
}

#[tokio::test]
async fn nicknames_follow_the_rule_and_cannot_copy_an_account() {
    let app = app_with(5).await;
    let (pair, csrf, _) = session(&app, None).await;
    let pair = pair.unwrap();
    let response = start(&app, &pair, &csrf, "a b").await;
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    assert_eq!(json_body(response).await["error"], "nickname_invalid");
    let response = start(&app, &pair, &csrf, "ANA").await;
    assert_eq!(response.status(), StatusCode::CONFLICT);
    assert_eq!(json_body(response).await["error"], "nickname_taken");
}

#[tokio::test]
async fn guest_starts_are_rate_limited_per_address() {
    let app = app_with(2).await;
    let (pair, csrf, _) = session(&app, None).await;
    let pair = pair.unwrap();
    // A refused CSRF check doesn't use up the allowance.
    assert_eq!(
        start(&app, &pair, "wrong", "Visitor1").await.status(),
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        start(&app, &pair, &csrf, "a b").await.status(),
        StatusCode::BAD_REQUEST
    );
    assert_eq!(
        start(&app, &pair, &csrf, "c d").await.status(),
        StatusCode::BAD_REQUEST
    );
    let response = start(&app, &pair, &csrf, "Visitor1").await;
    assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);
    assert_eq!(json_body(response).await["error"], "rate_limited");
}
