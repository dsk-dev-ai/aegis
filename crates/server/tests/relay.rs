use std::time::Duration;

use aegis_core::RelayEnvelope;
use aegis_server::{app, AppState};
use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpListener;
use tokio_tungstenite::{connect_async, tungstenite::Message};

type Ws = tokio_tungstenite::WebSocketStream<
    tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
>;

async fn connect(url: &str, token: &str) -> Ws {
    let (mut ws, _) = connect_async(url).await.expect("connect");
    ws.send(Message::Text(format!("{{\"hello\":\"{token}\"}}")))
        .await
        .expect("hello");
    ws
}

async fn recv_envelope(ws: &mut Ws) -> RelayEnvelope {
    let msg = tokio::time::timeout(Duration::from_secs(3), ws.next())
        .await
        .expect("no message within timeout")
        .expect("stream ended")
        .expect("ws error");
    match msg {
        Message::Text(text) => serde_json::from_str(&text).expect("envelope json"),
        other => panic!("expected text frame, got {other:?}"),
    }
}

#[tokio::test]
async fn routes_between_two_clients() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app(AppState::new())).await.unwrap();
    });
    let url = format!("ws://{addr}/v1/ws");

    let mut alice = connect(&url, "token-alice").await;
    let mut bob = connect(&url, "token-bob").await;

    let sent = RelayEnvelope::new("token-bob", b"ciphertext-for-bob");
    alice
        .send(Message::Text(serde_json::to_string(&sent).unwrap()))
        .await
        .unwrap();

    let received = recv_envelope(&mut bob).await;
    assert_eq!(received.to, "token-bob");
    assert_eq!(received.ciphertext().unwrap(), b"ciphertext-for-bob");
}

#[tokio::test]
async fn queues_when_offline_then_flushes_on_connect() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app(AppState::new())).await.unwrap();
    });
    let url = format!("ws://{addr}/v1/ws");

    // Bob is not connected yet.
    let mut alice = connect(&url, "token-alice").await;
    let sent = RelayEnvelope::new("token-bob", b"stored-while-offline");
    alice
        .send(Message::Text(serde_json::to_string(&sent).unwrap()))
        .await
        .unwrap();
    tokio::time::sleep(Duration::from_millis(150)).await;

    // Bob connects and should receive the queued envelope.
    let mut bob = connect(&url, "token-bob").await;
    let received = recv_envelope(&mut bob).await;
    assert_eq!(received.ciphertext().unwrap(), b"stored-while-offline");
}
