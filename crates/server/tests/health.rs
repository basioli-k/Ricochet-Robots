use std::sync::Arc;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use server::clock::FakeClock;
use server::{AppState, app};
use storage::InMemoryStore;
use tower::ServiceExt;

fn state() -> AppState {
    let store = Arc::new(InMemoryStore::new());
    AppState {
        players: store.clone(),
        games: store,
        clock: Arc::new(FakeClock::at(0)),
    }
}

#[tokio::test]
async fn health_answers_ok() {
    let response = app(state())
        .oneshot(Request::get("/health").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    assert_eq!(&body[..], b"ok");
}

#[tokio::test]
async fn unknown_paths_are_not_found() {
    let response = app(state())
        .oneshot(Request::get("/nope").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}
