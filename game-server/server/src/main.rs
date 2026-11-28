use axum::{
    Router,
    extract::{Query, State, ws::WebSocketUpgrade},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
};
use game_server::{
    client,
    constants::{CLIENT_MAX_MSG_SIZE, LOBBY_CHANNEL_SIZE, SERVER_LISTEN_HOST, SERVER_PORT_ENV},
    lobby::{InnerCommand, Lobby},
};
use serde::Deserialize;
use tokio::sync::mpsc;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let port: u16 = std::env::var(SERVER_PORT_ENV)
        .unwrap_or_else(|_| panic!("environment variable {SERVER_PORT_ENV} is not set"))
        .parse()
        .unwrap_or_else(|_| panic!("{SERVER_PORT_ENV} must be a valid port number (0-65535)"));
    let address = format!("{SERVER_LISTEN_HOST}:{port}");

    let (lobby_msg_sender, lobby_msg_receiver) = mpsc::channel(LOBBY_CHANNEL_SIZE);
    // start the lobby
    tokio::spawn(Lobby::new(lobby_msg_receiver).run());

    let app = Router::new()
        .route("/health", get(|| async { "ok\n" }))
        .route("/ws", get(websocket_upgrade))
        .with_state(lobby_msg_sender);

    let listener = tokio::net::TcpListener::bind(&address)
        .await
        .unwrap_or_else(|_| panic!("failed to bind server to {address}"));
    tracing::info!(
        "game-server listens on {}",
        listener.local_addr().expect("failed to get local address")
    );

    axum::serve(listener, app).await.expect("server closed");
}

/// temporary struct waiting for the admin server
#[derive(Deserialize)]
struct ConnectParams {
    user: String,
}

/// received an http upgrade to websocket
async fn websocket_upgrade(
    ws: WebSocketUpgrade,
    Query(params): Query<ConnectParams>, // ?arg= ... in the browser
    State(lobby): State<mpsc::Sender<InnerCommand>>, // given by the server
) -> Response {
    let user = params.user;
    if user.is_empty() || user.len() > 20 {
        return StatusCode::BAD_REQUEST.into_response();
    }
    ws.max_message_size(CLIENT_MAX_MSG_SIZE)
        .on_upgrade(move |socket| client::handle_client(socket, user, lobby))
}
