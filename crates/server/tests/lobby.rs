//! Opening and listing rooms through the HTTP API.

mod support;

use std::net::SocketAddr;

use axum::Router;
use axum::body::Body;
use axum::http::{Request, StatusCode, header};
use serde_json::{Value, json};
use server::identity::CSRF_HEADER;
use server::rooms::manager::RoomLimits;
use support::{Limits, app_with, apps_at, guest, json_body, session};
use tower::ServiceExt;

async fn create(app: &Router, cookie_pair: &str, csrf: &str) -> axum::response::Response {
    let request = Request::post("/api/rooms")
        .header(header::COOKIE, cookie_pair)
        .header(CSRF_HEADER, csrf)
        .body(Body::empty())
        .unwrap();
    app.clone().oneshot(request).await.unwrap()
}

async fn list(app: &Router) -> Value {
    let response = app
        .clone()
        .oneshot(Request::get("/api/rooms").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    json_body(response).await
}

fn rooms(global: usize, per_owner: usize, room_creates: usize) -> Limits {
    Limits {
        rooms: RoomLimits {
            global,
            per_owner,
            capacity: 8,
        },
        room_creates,
        ..Limits::default()
    }
}

#[tokio::test]
async fn a_guest_opens_a_room_and_the_lobby_lists_it() {
    let app = app_with(Limits::default()).await;
    assert_eq!(list(&app).await, json!([]));
    let (pair, csrf, _) = guest(&app, "Visitor1").await;
    let response = create(&app, &pair, &csrf).await;
    assert_eq!(response.status(), StatusCode::CREATED);
    let id = json_body(response).await["id"].clone();
    assert_eq!(id.as_str().map(str::len), Some(32), "128 random bits");
    assert_eq!(list(&app).await, json!([{ "id": id, "members": 0 }]));
}

#[tokio::test]
async fn opening_a_room_needs_a_csrf_token_and_a_player() {
    let app = app_with(Limits::default()).await;
    let (pair, csrf, _) = session(&app, None).await;
    let pair = pair.unwrap();
    let response = create(&app, &pair, &csrf).await;
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(json_body(response).await["error"], "not_signed_in");

    let (pair, _, _) = guest(&app, "Visitor1").await;
    let response = create(&app, &pair, "wrong").await;
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
    assert_eq!(json_body(response).await["error"], "csrf_mismatch");
    assert_eq!(list(&app).await, json!([]));
}

fn at(ip: &str) -> SocketAddr {
    SocketAddr::new(ip.parse().unwrap(), 4000)
}

#[tokio::test]
async fn a_player_is_capped_from_any_client() {
    let (apps, _) = apps_at(rooms(10, 1, 100), &[at("198.51.100.1"), at("198.51.100.2")]).await;
    let (pair, csrf, _) = guest(&apps[0], "Visitor1").await;
    assert_eq!(
        create(&apps[0], &pair, &csrf).await.status(),
        StatusCode::CREATED
    );
    let response = create(&apps[1], &pair, &csrf).await;
    assert_eq!(response.status(), StatusCode::CONFLICT);
    assert_eq!(json_body(response).await["error"], "room_limit");
}

#[tokio::test]
async fn a_client_is_capped_across_sessions_and_its_ipv6_64() {
    let (apps, _) = apps_at(
        rooms(10, 1, 100),
        &[at("2001:db8:1:2::1"), at("2001:db8:1:2::2")],
    )
    .await;
    let (first, first_csrf, _) = guest(&apps[0], "Visitor1").await;
    let (second, second_csrf, _) = guest(&apps[1], "Visitor2").await;
    assert_eq!(
        create(&apps[0], &first, &first_csrf).await.status(),
        StatusCode::CREATED
    );
    let response = create(&apps[1], &second, &second_csrf).await;
    assert_eq!(response.status(), StatusCode::CONFLICT);
    assert_eq!(json_body(response).await["error"], "room_limit");
}

#[tokio::test]
async fn the_server_wide_cap_has_its_own_code() {
    let (apps, _) = apps_at(rooms(1, 5, 100), &[at("198.51.100.1"), at("198.51.100.2")]).await;
    let (a, a_csrf, _) = guest(&apps[0], "Visitor1").await;
    let (b, b_csrf, _) = guest(&apps[1], "Visitor2").await;
    assert_eq!(
        create(&apps[0], &a, &a_csrf).await.status(),
        StatusCode::CREATED
    );
    let response = create(&apps[1], &b, &b_csrf).await;
    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(json_body(response).await["error"], "too_many_rooms");
}

#[tokio::test]
async fn room_creation_is_rate_limited_per_client_across_sessions() {
    let app = app_with(rooms(10, 5, 1)).await;
    let (first, first_csrf, _) = guest(&app, "Visitor1").await;
    let (second, second_csrf, _) = guest(&app, "Visitor2").await;
    assert_eq!(
        create(&app, &first, &first_csrf).await.status(),
        StatusCode::CREATED
    );
    let response = create(&app, &second, &second_csrf).await;
    assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);
    assert_eq!(json_body(response).await["error"], "rate_limited");
}
