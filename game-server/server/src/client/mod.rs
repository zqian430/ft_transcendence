mod heartbeat;
mod msg_sender;

pub use msg_sender::ClientMsgSender;

use axum::{
    body::Bytes,
    extract::ws::{CloseFrame, Message, WebSocket},
};
use game_protocol::{ClientMsg, ServerMsg};
use tokio::{
    sync::{mpsc, oneshot},
    time::timeout,
};
use tracing::warn;

use crate::{
    client::heartbeat::Heartbeat,
    constants::{CLIENT_CHANNEL_SIZE, CLIENT_SEND_TIMEOUT, ERR_ALREADY_CONNECTED},
    lobby::{InnerCommand, UserId},
};

/// all client logic
pub async fn handle_client(mut socket: WebSocket, user: UserId, lobby: mpsc::Sender<InnerCommand>) {
    let (client_accepted_sender, client_accepted_receiver) = oneshot::channel();
    let (sender, mut msg_receiver) = mpsc::channel::<ServerMsg>(CLIENT_CHANNEL_SIZE); // all messages sent to clients
    let msg_sender = ClientMsgSender::new(sender); // to send messages to clients directly from this function
    if lobby
        .send(InnerCommand::Connect {
            user: user.clone(),
            msg_sender: msg_sender.clone(),
            client_accepted_sender,
        })
        .await
        .is_err()
    {
        return; // lobby is dead
    }

    if client_accepted_receiver.await != Ok(true) {
        let close = Message::Close(Some(CloseFrame {
            code: ERR_ALREADY_CONNECTED,
            reason: "user already connected".into(),
        }));
        send_websocket_msg(&mut socket, close).await;
        return; // the lobby refused
    }

    let mut heartbeat = Heartbeat::new();

    loop {
        tokio::select! {
            //the client sent us a message
            incoming_msg = socket.recv() => {
                let Some(Ok(client_msg)) = incoming_msg else {break};
                heartbeat.seen();
                match client_msg {
                    Message::Text(text) => match serde_json::from_str::<ClientMsg>(&text) {
                        Ok(msg) => {
                            if lobby.send(InnerCommand::FromClient { from: user.clone(), msg }).await.is_err() {
                                break; // lobby is dead
                            }
                        }
                        Err(e) => {
                            warn!(%user, %e, "invalid command");
                            msg_sender.send_error(format!("invalid command: {}", text.as_str()));
                        }
                    },

                    Message::Close(_) => break,
                    _ => {}
                }
            }

            // check that the client is still alive and ping it
            _ = heartbeat.wait_tick() => {
                if !heartbeat.is_alive() {
                    warn!(%user, "heartbeat timeout");
                    break;
                }
                if !send_websocket_msg(&mut socket, Message::Ping(Bytes::new())).await {
                    break;
                }
            }

            // a part of the server wants to send a message to a client
            outgoing = msg_receiver.recv() => {
                let Some(msg) = outgoing else { break };
                let json = serde_json::to_string(&msg).expect("ServerMsg is always serializable");
                if !send_websocket_msg(&mut socket, Message::Text(json.into())).await {
                    break;
                }
            }
        }
    }
    let _ = lobby.send(InnerCommand::Disconnect { user }).await;
}

/// sends a websocket message, returns false if the socket is closed or blocked for too long
async fn send_websocket_msg(socket: &mut WebSocket, msg: Message) -> bool {
    let result = timeout(CLIENT_SEND_TIMEOUT, socket.send(msg)).await;
    matches!(result, Ok(Ok(())))
}
