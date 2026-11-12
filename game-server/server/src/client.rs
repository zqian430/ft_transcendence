use axum::extract::ws::{Message, WebSocket};
use engine::protocol::ClientMsg;
use tracing::{info, warn};

/// a websocket connection. all client logic is here
pub async fn handle_client(mut socket: WebSocket) {
    tracing::info!("client connected");

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

    tracing::info!("client disconnected");
}
