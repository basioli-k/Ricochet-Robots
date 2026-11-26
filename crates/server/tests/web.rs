//! Serving the built web client next to the API.

mod support;

use std::path::PathBuf;

use axum::body::Body;
use axum::http::{Request, StatusCode, header};
use http_body_util::BodyExt;
use server::web::{CONTENT_SECURITY_POLICY, with_web};
use support::{Limits, app_with};
use tower::ServiceExt;

/// A throwaway build (an index and one hashed asset), removed on drop even
/// when an assertion fails. A secret file sits beside it, outside the build.
struct Build(PathBuf);

impl Build {
    fn new(name: &str) -> Self {
        let root = std::env::temp_dir().join(format!("rr-web-{name}-{}", std::process::id()));
        let dir = root.join("dist");
        std::fs::create_dir_all(dir.join("assets")).unwrap();
        std::fs::write(dir.join("index.html"), "<!doctype html><title>RR</title>").unwrap();
        std::fs::write(dir.join("assets/app-1a2b.js"), "console.log(1)").unwrap();
        std::fs::write(root.join("secret.txt"), "not for the web").unwrap();
        Self(root)
    }

    fn dir(&self) -> PathBuf {
        self.0.join("dist")
    }
}

impl Drop for Build {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

async fn get(app: &axum::Router, path: &str) -> axum::response::Response {
    app.clone()
        .oneshot(Request::get(path).body(Body::empty()).unwrap())
        .await
        .unwrap()
}

#[tokio::test]
async fn the_index_and_assets_are_served_with_their_cache_and_security_headers() {
    let build = Build::new("serve");
    let app = with_web(app_with(Limits::default()).await, &build.dir());

    let index = get(&app, "/").await;
    assert_eq!(index.status(), StatusCode::OK);
    assert_eq!(index.headers()[header::CACHE_CONTROL], "no-cache");
    assert_eq!(
        index.headers()[header::CONTENT_SECURITY_POLICY],
        CONTENT_SECURITY_POLICY
    );
    assert_eq!(index.headers()[header::X_CONTENT_TYPE_OPTIONS], "nosniff");
    assert_eq!(index.headers()[header::REFERRER_POLICY], "same-origin");
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
}

#[tokio::test]
async fn unknown_paths_and_escapes_are_not_found() {
    let build = Build::new("missing");
    let app = with_web(app_with(Limits::default()).await, &build.dir());
    // No fallback to the index: the client has no routes of its own.
    let page = get(&app, "/lobby").await;
    assert_eq!(page.status(), StatusCode::NOT_FOUND);
    assert_eq!(page.headers()[header::CACHE_CONTROL], "no-cache");
    assert_eq!(
        page.headers()[header::CONTENT_SECURITY_POLICY],
        CONTENT_SECURITY_POLICY
    );
    for escape in [
        "/../secret.txt",
        "/..%2fsecret.txt",
        "/assets/..%2f..%2fsecret.txt",
    ] {
        let response = get(&app, escape).await;
        assert_ne!(response.status(), StatusCode::OK, "{escape}");
        let body = response.into_body().collect().await.unwrap().to_bytes();
        assert!(!body.starts_with(b"not for the web"), "{escape}");
    }
}

#[tokio::test]
async fn the_api_keeps_its_routes() {
    let build = Build::new("api");
    let app = with_web(app_with(Limits::default()).await, &build.dir());
    let session = get(&app, "/api/session").await;
    assert_eq!(session.status(), StatusCode::OK);
    assert!(
        session
            .headers()
            .get(header::CONTENT_TYPE)
            .is_some_and(|v| v.to_str().unwrap().starts_with("application/json"))
    );
    assert_eq!(get(&app, "/health").await.status(), StatusCode::OK);
}
