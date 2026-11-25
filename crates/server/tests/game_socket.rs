//! Playing over the game socket with real WebSocket clients.

mod support;

use std::net::SocketAddr;
use std::time::Duration;

use axum::Router;
use futures_util::{SinkExt, StreamExt};
use serde_json::{Value, json};
use server::rooms::state::BIDDING_WINDOW_MS;
use support::{CLIENT, Limits, ORIGIN, apps_at, create_room, guest};
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::{self, Message};
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};

type Client = WebSocketStream<MaybeTlsStream<TcpStream>>;

/// Serves `app` on a free local port.
async fn serve(app: Router) -> SocketAddr {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    addr
}

fn request(
    addr: SocketAddr,
    room: &str,
    cookie: Option<&str>,
    origin: &str,
) -> tungstenite::handshake::client::Request {
    let mut request = format!("ws://{addr}/api/rooms/{room}/ws")
        .into_client_request()
        .unwrap();
    if let Some(cookie) = cookie {
        request
            .headers_mut()
            .insert("cookie", cookie.parse().unwrap());
    }
    request
        .headers_mut()
        .insert("origin", origin.parse().unwrap());
    request
}

async fn open(
    addr: SocketAddr,
    room: &str,
    cookie: Option<&str>,
    origin: &str,
) -> Result<Client, tungstenite::Error> {
    tokio_tungstenite::connect_async(request(addr, room, cookie, origin))
        .await
        .map(|(client, _)| client)
}

/// The HTTP status a refused upgrade answered with.
async fn refused(addr: SocketAddr, room: &str, cookie: Option<&str>, origin: &str) -> u16 {
    match open(addr, room, cookie, origin).await {
        Err(tungstenite::Error::Http(response)) => response.status().as_u16(),
        other => panic!("expected a refusal, got {:?}", other.map(|_| ())),
    }
}

/// The next JSON message matching `wanted`, skipping control frames and
/// other messages; fails if none arrives within 2 s overall.
async fn recv_where(client: &mut Client, wanted: impl Fn(&Value) -> bool) -> Value {
    let deadline = tokio::time::Instant::now() + Duration::from_secs(2);
    loop {
        let frame = tokio::time::timeout_at(deadline, client.next())
            .await
            .expect("the message within 2 s")
            .expect("the socket is open")
            .unwrap();
        if let Message::Text(text) = frame {
            let message = serde_json::from_str(&text).unwrap();
            if wanted(&message) {
                return message;
            }
        }
    }
}

/// The next JSON message.
async fn recv(client: &mut Client) -> Value {
    recv_where(client, |_| true).await
}

/// The next message of type `kind`.
async fn recv_type(client: &mut Client, kind: &str) -> Value {
    recv_where(client, |m| m["type"] == kind).await
}

async fn send(client: &mut Client, message: Value) {
    client
        .send(Message::Text(message.to_string().into()))
        .await
        .unwrap();
}

struct Table {
    addr: SocketAddr,
    clock: std::sync::Arc<server::clock::FakeClock>,
    room: String,
    /// (cookie, player id) for "Ana1" and "Bob1".
    ana: (String, String),
    bob: (String, String),
}

/// Two guests and a room Ana opened, served on a real port.
async fn table(limits: Limits) -> Table {
    let (mut apps, clock) = apps_at(limits, &[SocketAddr::from((CLIENT, 4000))]).await;
    let app = apps.remove(0);
    let (ana_cookie, ana_csrf, ana_id) = guest(&app, "Ana1").await;
    let (bob_cookie, _, bob_id) = guest(&app, "Bob1").await;
    let room = create_room(&app, &ana_cookie, &ana_csrf).await;
    Table {
        addr: serve(app).await,
        clock,
        room,
        ana: (ana_cookie, ana_id),
        bob: (bob_cookie, bob_id),
    }
}

impl Table {
    async fn join(&self, who: &(String, String)) -> (Client, Value) {
        let mut client = open(self.addr, &self.room, Some(&who.0), ORIGIN)
            .await
            .unwrap();
        let snapshot = recv(&mut client).await;
        assert_eq!(snapshot["type"], "snapshot");
        assert_eq!(snapshot["you"], who.1.as_str());
        (client, snapshot)
    }
}

#[tokio::test]
async fn two_guests_play_a_round() {
    let t = table(Limits::default()).await;
    let (mut ana, _) = t.join(&t.ana).await;
    let (mut bob, snapshot) = t.join(&t.bob).await;
    assert_eq!(
        snapshot["room"]["members"],
        json!([
            { "id": t.ana.1, "name": "Ana1", "is_guest": true },
            { "id": t.bob.1, "name": "Bob1", "is_guest": true }
        ])
    );
    assert_eq!(
        recv_type(&mut ana, "joined").await["player"]["id"],
        t.bob.1.as_str()
    );

    send(&mut ana, json!({ "type": "start" })).await;
    for client in [&mut ana, &mut bob] {
        recv_type(client, "round_started").await;
    }
    send(&mut bob, json!({ "type": "bid", "moves": 3 })).await;
    for client in [&mut ana, &mut bob] {
        let bid = recv_type(client, "bid_placed").await;
        assert_eq!(
            (bid["player"].as_str(), bid["moves"].as_u64()),
            (Some(t.bob.1.as_str()), Some(3))
        );
    }
    // The room's own ticker closes bidding once the window has passed.
    t.clock.advance(BIDDING_WINDOW_MS);
    for client in [&mut ana, &mut bob] {
        let demo = recv_type(client, "demonstration_started").await;
        assert_eq!(demo["player"], t.bob.1.as_str());
    }
    send(&mut ana, json!({ "type": "concede" })).await;
    assert_eq!(
        recv(&mut ana).await,
        json!({ "type": "error", "code": "not_your_turn" })
    );
    send(&mut bob, json!({ "type": "concede" })).await;
    for client in [&mut ana, &mut bob] {
        let resolved = recv_type(client, "round_resolved").await;
        assert_eq!(resolved["phase"], json!({ "kind": "unsolved" }));
    }
}

#[tokio::test]
async fn only_signed_in_players_from_our_origin_reach_a_room() {
    let t = table(Limits::default()).await;
    let cookie = Some(t.ana.0.as_str());
    assert_eq!(
        refused(t.addr, &t.room, cookie, "https://evil.example").await,
        403
    );
    let mut no_origin = request(t.addr, &t.room, cookie, ORIGIN);
    no_origin.headers_mut().remove("origin");
    match tokio_tungstenite::connect_async(no_origin).await {
        Err(tungstenite::Error::Http(response)) => assert_eq!(response.status().as_u16(), 403),
        other => panic!("expected a refusal, got {:?}", other.map(|_| ())),
    }
    assert_eq!(refused(t.addr, &t.room, None, ORIGIN).await, 401);
    assert_eq!(refused(t.addr, "no-such-room", cookie, ORIGIN).await, 404);
}

#[tokio::test]
async fn bad_messages_and_floods_are_refused_by_code() {
    let t = table(Limits {
        chats: 1,
        ..Limits::default()
    })
    .await;
    let (mut ana, _) = t.join(&t.ana).await;
    send(&mut ana, json!({ "type": "fly" })).await;
    assert_eq!(recv_type(&mut ana, "error").await["code"], "bad_message");
    ana.send(Message::Binary(vec![1, 2].into())).await.unwrap();
    assert_eq!(recv_type(&mut ana, "error").await["code"], "bad_message");
    send(&mut ana, json!({ "type": "bid", "moves": 3 })).await;
    assert_eq!(recv_type(&mut ana, "error").await["code"], "not_started");

    send(&mut ana, json!({ "type": "chat", "text": "hi" })).await;
    let chat = recv_type(&mut ana, "chat").await;
    assert_eq!(
        (chat["player"].as_str(), chat["text"].as_str()),
        (Some(t.ana.1.as_str()), Some("hi"))
    );
    send(&mut ana, json!({ "type": "chat", "text": "again" })).await;
    assert_eq!(recv_type(&mut ana, "error").await["code"], "rate_limited");
    let (mut bob, _) = t.join(&t.bob).await;
    send(&mut bob, json!({ "type": "chat", "text": "mine" })).await;
    assert_eq!(
        recv_type(&mut bob, "chat").await["text"],
        "mine",
        "per player"
    );
    t.clock.advance(1_000);
    send(&mut ana, json!({ "type": "chat", "text": "later" })).await;
    recv_where(&mut ana, |m| m["type"] == "chat" && m["text"] == "later").await;
}

#[tokio::test]
async fn bids_and_moves_are_rate_limited_per_player() {
    let t = table(Limits {
        actions: 2,
        ..Limits::default()
    })
    .await;
    let (mut ana, _) = t.join(&t.ana).await;
    send(&mut ana, json!({ "type": "start" })).await;
    recv_type(&mut ana, "round_started").await;
    send(&mut ana, json!({ "type": "bid", "moves": 9 })).await;
    recv_type(&mut ana, "bid_placed").await;
    // Starting used one of Ana's two actions this second, the bid the other.
    send(&mut ana, json!({ "type": "bid", "moves": 8 })).await;
    assert_eq!(recv_type(&mut ana, "error").await["code"], "rate_limited");
    send(&mut ana, json!({ "type": "fly" })).await;
    assert_eq!(
        recv_type(&mut ana, "error").await["code"],
        "rate_limited",
        "garbage counts too"
    );
    // The limit is Ana's own: Bob can still bid.
    let (mut bob, _) = t.join(&t.bob).await;
    send(&mut bob, json!({ "type": "bid", "moves": 7 })).await;
    assert_eq!(recv_type(&mut bob, "bid_placed").await["moves"], 7);
}

#[tokio::test]
async fn closing_or_leaving_takes_a_player_out() {
    let t = table(Limits::default()).await;
    let (mut ana, _) = t.join(&t.ana).await;
    let (bob, _) = t.join(&t.bob).await;
    recv_type(&mut ana, "joined").await;
    drop(bob);
    assert_eq!(
        recv_type(&mut ana, "left").await["player"],
        t.bob.1.as_str()
    );

    let (mut bob, _) = t.join(&t.bob).await;
    recv_type(&mut ana, "joined").await;
    send(&mut bob, json!({ "type": "leave" })).await;
    assert_eq!(
        recv_type(&mut ana, "left").await["player"],
        t.bob.1.as_str()
    );
}

#[tokio::test]
async fn a_silent_client_is_dropped_after_the_heartbeat_timeout() {
    let t = table(Limits::default()).await;
    let (mut ana, _) = t.join(&t.ana).await;
    // Bob never reads again, so he never answers the server's pings.
    let (_bob, _) = t.join(&t.bob).await;
    recv_type(&mut ana, "joined").await;
    // The 1 s timeout plus a ping interval, with room to spare.
    let left = tokio::time::timeout(
        Duration::from_secs(5),
        recv_where_patiently(&mut ana, |m| m["type"] == "left"),
    )
    .await
    .expect("Bob dropped within 5 s");
    assert_eq!(left["player"], t.bob.1.as_str());
}

#[tokio::test]
async fn a_full_room_refuses_by_code() {
    let t = table(Limits {
        rooms: server::rooms::manager::RoomLimits {
            capacity: 1,
            ..server::rooms::manager::RoomLimits::default()
        },
        ..Limits::default()
    })
    .await;
    let (_ana, _) = t.join(&t.ana).await;
    let mut bob = open(t.addr, &t.room, Some(&t.bob.0), ORIGIN).await.unwrap();
    assert_eq!(
        recv(&mut bob).await,
        json!({ "type": "error", "code": "room_full" })
    );
}

#[tokio::test]
async fn a_player_can_hold_only_so_many_sockets() {
    // Both players connect from one client, whose cap stays out of the way.
    let t = table(Limits {
        sockets_per_player: 1,
        ..Limits::default()
    })
    .await;
    let (ana, _) = t.join(&t.ana).await;
    let cookie = Some(t.ana.0.as_str());
    assert_eq!(refused(t.addr, &t.room, cookie, ORIGIN).await, 429);
    let _bob = t.join(&t.bob).await;
    drop(ana);
    // The place frees once the server notices the close.
    let deadline = tokio::time::Instant::now() + Duration::from_secs(2);
    while open(t.addr, &t.room, cookie, ORIGIN).await.is_err() {
        assert!(tokio::time::Instant::now() < deadline, "still refused");
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
}

#[tokio::test]
async fn an_oversized_message_closes_the_socket() {
    let t = table(Limits::default()).await;
    let (mut ana, _) = t.join(&t.ana).await;
    let (mut bob, _) = t.join(&t.bob).await;
    recv_type(&mut ana, "joined").await;
    let huge = json!({ "type": "chat", "text": "x".repeat(5_000) });
    let _ = bob.send(Message::Text(huge.to_string().into())).await;
    assert_eq!(
        recv_type(&mut ana, "left").await["player"],
        t.bob.1.as_str()
    );
}

#[tokio::test]
async fn a_client_that_stops_reading_is_dropped() {
    let t = table(Limits {
        chats: 1_000_000,
        ..Limits::default()
    })
    .await;
    let (mut ana, _) = t.join(&t.ana).await;
    // Bob's tiny receive buffer fills fast once he stops reading.
    let socket = tokio::net::TcpSocket::new_v4().unwrap();
    socket.set_recv_buffer_size(1024).unwrap();
    let stream = MaybeTlsStream::Plain(socket.connect(t.addr).await.unwrap());
    let (mut bob, _) =
        tokio_tungstenite::client_async(request(t.addr, &t.room, Some(&t.bob.0), ORIGIN), stream)
            .await
            .unwrap();
    assert_eq!(recv(&mut bob).await["type"], "snapshot");
    recv_type(&mut ana, "joined").await;
    let text = "y".repeat(200);
    let deadline = tokio::time::Instant::now() + Duration::from_secs(20);
    loop {
        assert!(
            tokio::time::Instant::now() < deadline,
            "Bob was never dropped"
        );
        send(&mut ana, json!({ "type": "chat", "text": text })).await;
        let next = recv_where(&mut ana, |m| m["type"] == "chat" || m["type"] == "left").await;
        if next["type"] == "left" {
            assert_eq!(next["player"], t.bob.1.as_str());
            break;
        }
    }
}

#[tokio::test]
async fn a_second_socket_keeps_the_player_in_the_room() {
    let t = table(Limits::default()).await;
    let (mut ana, _) = t.join(&t.ana).await;
    let (first, _) = t.join(&t.bob).await;
    recv_type(&mut ana, "joined").await;
    let (mut second, snapshot) = t.join(&t.bob).await;
    assert_eq!(snapshot["room"]["members"].as_array().unwrap().len(), 2);
    drop(first);
    // Give the server time to notice the close.
    tokio::time::sleep(Duration::from_millis(200)).await;
    // Bob is still there: his chat arrives, and no "left" came before it.
    send(&mut second, json!({ "type": "chat", "text": "still here" })).await;
    let next = recv_where(&mut ana, |m| m["type"] == "chat" || m["type"] == "left").await;
    assert_eq!(next["type"], "chat");
}

/// Like `recv_where`, without its 2 s deadline (the caller sets one).
async fn recv_where_patiently(client: &mut Client, wanted: impl Fn(&Value) -> bool) -> Value {
    loop {
        match client.next().await {
            Some(Ok(Message::Text(text))) => {
                let message: Value = serde_json::from_str(&text).unwrap();
                if wanted(&message) {
                    return message;
                }
            }
            Some(Ok(_)) => {}
            other => panic!("the socket ended: {other:?}"),
        }
    }
}
