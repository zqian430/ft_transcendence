use axum::{Router, extract::ws::WebSocketUpgrade, response::Response, routing::get};

use server::{
    client,
    constants::{CLIENT_MAX_MSG_SIZE, SERVER_LISTEN_HOST, SERVER_PORT_ENV},
};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let port: u16 = std::env::var(SERVER_PORT_ENV)
        .unwrap_or_else(|_| panic!("environment variable {SERVER_PORT_ENV} is not set"))
        .parse()
        .unwrap_or_else(|_| panic!("{SERVER_PORT_ENV} must be a valid port number (0-65535)"));
    let address = format!("{SERVER_LISTEN_HOST}:{port}");

    let app = Router::new()
        .route("/health", get(|| async { "ok\n" }))
        .route("/ws", get(websocket_upgrade));

    let listener = tokio::net::TcpListener::bind(&address)
        .await
        .unwrap_or_else(|_| panic!("failed to bind server to {address}"));
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
