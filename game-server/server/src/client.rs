use axum::extract::ws::{Message, WebSocket};
use game_protocol::{ClientMsg, ServerMsg};
use tokio::sync::mpsc;
use tracing::{info, warn};

use crate::constants::CLIENT_CHANNEL_SIZE;

/// a websocket connection. all client logic is here
pub async fn handle_client(mut socket: WebSocket) {
    // client receives msgs here
    let (msg_sender, mut msg_receiver) = mpsc::channel::<ServerMsg>(CLIENT_CHANNEL_SIZE);
    if coordinator.send(Command::Connect { user: user.clone(), tx }).await.is_err() {
        return; // coordinator is dead
    }
    while let Some(Ok(msg)) = socket.recv().await {
        match msg {
            Message::Text(text) => {
                match serde_json::from_str::<ClientMsg>(&text) {
                    Ok(parsed) => info!(?parsed, "received message"),
                    Err(e) => warn!(%e, "ignored invalid message"),
                }

                //send back for debugging
                if socket.send(Message::Text(text)).await.is_err() {
                    break;
                }
            }

            // todo: heartbeat because close
            // does not handle the case of an internet disconnection
            Message::Close(_) => break,
            _ => {}
        }
    }
}
