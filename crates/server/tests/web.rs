//! Serving the built web client next to the API.

mod support;

use std::path::PathBuf;

use axum::body::Body;
use axum::http::{Request, StatusCode, header};
use http_body_util::BodyExt;
use server::web::{CONTENT_SECURITY_POLICY, with_web};
use support::{Limits, app_with};
use tower::ServiceExt;

/// A throwaway build: an index and one hashed asset.
fn build_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("rr-web-{name}-{}", std::process::id()));
    std::fs::create_dir_all(dir.join("assets")).unwrap();
    std::fs::write(dir.join("index.html"), "<!doctype html><title>RR</title>").unwrap();
    std::fs::write(dir.join("assets/app-1a2b.js"), "console.log(1)").unwrap();
    dir
}

async fn get(app: &axum::Router, path: &str) -> axum::response::Response {
    app.clone()
        .oneshot(Request::get(path).body(Body::empty()).unwrap())
        .await
        .unwrap()
}

#[tokio::test]
async fn the_index_and_assets_are_served_with_their_cache_and_security_headers() {
    let dir = build_dir("serve");
    let app = with_web(app_with(Limits::default()).await, &dir);

    let index = get(&app, "/").await;
    assert_eq!(index.status(), StatusCode::OK);
    assert_eq!(index.headers()[header::CACHE_CONTROL], "no-cache");
    assert_eq!(
        index.headers()[header::CONTENT_SECURITY_POLICY],
        CONTENT_SECURITY_POLICY
    );
    assert_eq!(index.headers()[header::X_CONTENT_TYPE_OPTIONS], "nosniff");
    let body = index.into_body().collect().await.unwrap().to_bytes();
    assert!(body.starts_with(b"<!doctype html>"));

    let asset = get(&app, "/assets/app-1a2b.js").await;
    assert_eq!(asset.status(), StatusCode::OK);
    assert!(
        asset.headers()[header::CACHE_CONTROL]
            .to_str()
            .unwrap()
            .contains("immutable")
    );

    let missing = get(&app, "/assets/gone.js").await;
    assert_eq!(missing.status(), StatusCode::NOT_FOUND);
    assert_eq!(missing.headers()[header::CACHE_CONTROL], "no-cache");
    std::fs::remove_dir_all(dir).unwrap();
}

#[tokio::test]
async fn the_api_keeps_its_routes() {
    let dir = build_dir("api");
    let app = with_web(app_with(Limits::default()).await, &dir);
    let session = get(&app, "/api/session").await;
    assert_eq!(session.status(), StatusCode::OK);
    assert!(
        session
            .headers()
            .get(header::CONTENT_TYPE)
            .is_some_and(|v| v.to_str().unwrap().starts_with("application/json"))
    );
    assert_eq!(get(&app, "/health").await.status(), StatusCode::OK);
    std::fs::remove_dir_all(dir).unwrap();
}
