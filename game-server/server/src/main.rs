use axum::{
    Router,
    extract::ws::WebSocketUpgrade,
    response::Response,
    routing::get,
};

use tracing;

use server::{client, constants::{CLIENT_MAX_MSG_SIZE, SERVER_LISTEN_ADRESS}};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let app = Router::new()
        .route("/health", get(|| async { "ok\n" }))
        .route("/ws", get(websocket_upgrade));

    let listener = tokio::net::TcpListener::bind(SERVER_LISTEN_ADRESS)
        .await
        .unwrap();
    tracing::info!("game-server listens on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

/// received an http upgrade to websocket
async fn websocket_upgrade(ws: WebSocketUpgrade) -> Response {
    ws.max_message_size(CLIENT_MAX_MSG_SIZE)
        .on_upgrade(client::handle_client)
}

