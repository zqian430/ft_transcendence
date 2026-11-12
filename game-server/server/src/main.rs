use axum::{Router, extract::ws::WebSocketUpgrade, response::Response, routing::get};

use server::{
    client,
    constants::{CLIENT_MAX_MSG_SIZE, SERVER_LISTEN_ADDRESS},
};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let app = Router::new()
        .route("/health", get(|| async { "ok\n" }))
        .route("/ws", get(websocket_upgrade));

    let listener = tokio::net::TcpListener::bind(SERVER_LISTEN_ADDRESS)
        .await
        .expect(format!("failed to bind server to {}", SERVER_LISTEN_ADDRESS).as_str());
    tracing::info!(
        "game-server listens on {}",
        listener.local_addr().expect("failed to get local address")
    );
    axum::serve(listener, app).await.expect("server closed");
}

/// received an http upgrade to websocket
async fn websocket_upgrade(ws: WebSocketUpgrade) -> Response {
    ws.max_message_size(CLIENT_MAX_MSG_SIZE)
        .on_upgrade(client::handle_client)
}
