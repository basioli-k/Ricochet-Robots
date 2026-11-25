//! Starting a guest session through the HTTP API.

mod support;

use std::net::SocketAddr;

use axum::body::Body;
use axum::http::{Request, StatusCode, header};
use serde_json::{Value, json};
use support::{CLIENT, Limits, apps_at, cookie, json_body, session, start};
use tower::ServiceExt;

/// An app allowing `guest_starts` guest starts per client per minute.
async fn app_with(guest_starts: usize) -> axum::Router {
    support::app_with(Limits {
        guest_starts,
        ..Limits::default()
    })
    .await
}

fn limit(guest_starts: usize) -> Limits {
    Limits {
        guest_starts,
        ..Limits::default()
    }
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
    let pair = pair.unwrap();
    let response = start(&app, &pair, "wrong", "Visitor1").await;
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
    assert_eq!(json_body(response).await["error"], "csrf_mismatch");
    let no_header = Request::post("/api/guest")
        .header(header::COOKIE, &pair)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(json!({ "nickname": "Visitor1" }).to_string()))
        .unwrap();
    let response = app.clone().oneshot(no_header).await.unwrap();
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
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
    let (apps, clock) = apps_at(limit(2), &[SocketAddr::from((CLIENT, 4000))]).await;
    let app = &apps[0];
    let (pair, csrf, _) = session(app, None).await;
    let pair = pair.unwrap();
    // Neither a refused CSRF check nor a malformed name uses the allowance.
    assert_eq!(
        start(app, &pair, "wrong", "Visitor1").await.status(),
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        start(app, &pair, &csrf, "a b").await.status(),
        StatusCode::BAD_REQUEST
    );
    // Well-formed attempts count, even when the name turns out taken.
    for _ in 0..2 {
        assert_eq!(
            start(app, &pair, &csrf, "ANA").await.status(),
            StatusCode::CONFLICT
        );
    }
    let response = start(app, &pair, &csrf, "Visitor1").await;
    assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);
    assert_eq!(json_body(response).await["error"], "rate_limited");
    clock.advance(60_000);
    assert_eq!(
        start(app, &pair, &csrf, "Visitor1").await.status(),
        StatusCode::OK,
        "the window reopens"
    );
}

#[tokio::test]
async fn ipv6_clients_share_the_limit_of_their_64_prefix() {
    let at = |ip: &str| SocketAddr::new(ip.parse().unwrap(), 4000);
    let (apps, _) = apps_at(
        limit(1),
        &[
            at("2001:db8:1:2::1"),
            at("2001:db8:1:2::2"),
            at("2001:db8:1:3::1"),
        ],
    )
    .await;
    let mut statuses = Vec::new();
    for app in &apps {
        let (pair, csrf, _) = session(app, None).await;
        statuses.push(start(app, &pair.unwrap(), &csrf, "Visitor1").await.status());
    }
    assert_eq!(
        statuses,
        [
            StatusCode::OK,
            StatusCode::TOO_MANY_REQUESTS,
            StatusCode::OK
        ],
        "a fresh address in the same /64 doesn't reset the limit"
    );
}
