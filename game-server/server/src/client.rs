use axum::extract::ws::{CloseFrame, Message, WebSocket};
use game_protocol::{ClientMsg, ServerMsg};
use tokio::sync::{mpsc, oneshot};
use tracing::warn;

use crate::{
    constants::{CLIENT_CHANNEL_SIZE, ERR_ALREADY_CONNECTED}, coordinator::{InnerCommand, UserId},
};

/// all client logic
pub async fn handle_client(
    mut socket: WebSocket,
    user: UserId,
    coordinator: mpsc::Sender<InnerCommand>,
) {
    let (client_accepted_sender, client_accepted_receiver) = oneshot::channel();
    // client receives msgs here
    let (msg_sender, mut msg_receiver) = mpsc::channel::<ServerMsg>(CLIENT_CHANNEL_SIZE);
    if coordinator
        .send(InnerCommand::Connect {
            user: user.clone(),
            msg_sender,
            client_accepted_sender
        })
        .await
        .is_err()
    {
        return; // coordinator is dead
    }

    if client_accepted_receiver.await != Ok(true) {
        let _ = socket
            .send(Message::Close(Some(CloseFrame {
                code: ERR_ALREADY_CONNECTED,
                reason: "user already connected".into(),
            })))
            .await;
        return; // quit if the coordinator refuses 
    }

    loop {
        tokio::select! {
            //the client sent us a message
            incoming_msg = socket.recv() => {
                let Some(Ok(client_msg)) = incoming_msg else {break};
                match client_msg {
                    Message::Text(text) => match serde_json::from_str::<ClientMsg>(&text) {
                        Ok(ClientMsg::ChatGlobal { text }) => {
                            let _ = coordinator.send(InnerCommand::ChatGlobal { from: user.clone(), text }).await;
                        }
                        Ok(ClientMsg::Chat { to, text }) => {
                            let _ = coordinator.send(InnerCommand::Chat { from: user.clone(), to, text }).await;
                        }
                        Err(e) => {
                            warn!(%user, %e, "invalid command");
                            if !send_error(&mut socket, format!("invalid command: {}", text.as_str())).await {
                                break;
                            }
                        }
                    },

                    // todo: heartbeat because close
                    // does not handle the case of an internet disconnection
                    Message::Close(_) => break,
                    _ => {}
                }
            }

            // a part of the server wants to send a message to a client
            outgoing = msg_receiver.recv() => {
                let Some(msg) = outgoing else { break };
                if !send_msg(&mut socket, &msg).await {
                    break;
                }
            }
        }
    }
    let _ = coordinator.send(InnerCommand::Disconnect { user }).await;
}

/// serializes a message and sends it to the client, returns false if the socket is closed
async fn send_msg(socket: &mut WebSocket, msg: &ServerMsg) -> bool {
    let json = serde_json::to_string(msg).expect("ServerMsg is always serializable");
    socket.send(Message::Text(json.into())).await.is_ok()
}

/// sends an error message to the client, returns false if the socket is closed
async fn send_error(socket: &mut WebSocket, message: impl Into<String>) -> bool {
    send_msg(
        socket,
        &ServerMsg::Error {
            message: message.into(),
        },
    )
    .await
}
