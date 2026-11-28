mod heartbeat;
mod msg_sender;
mod tokens;

pub use msg_sender::ClientMsgSender;

use std::ops::ControlFlow;

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
    client::{heartbeat::Heartbeat, tokens::Tokens},
    constants::{CLIENT_CHANNEL_SIZE, CLIENT_SEND_TIMEOUT, ERR_ALREADY_CONNECTED},
    lobby::{InnerCommand, UserId},
};

/// all client logic
pub async fn handle_client(mut socket: WebSocket, user: UserId, lobby: mpsc::Sender<InnerCommand>) {
    let (sender, msg_receiver) = mpsc::channel::<ServerMsg>(CLIENT_CHANNEL_SIZE); // all messages sent to clients
    let msg_sender = ClientMsgSender::new(sender); // to send messages to clients directly from the client task

    if !join_lobby(&mut socket, &user, &lobby, msg_sender.clone()).await {
        return;
    }
    Client {
        socket,
        user,
        lobby,
        msg_sender,
        msg_receiver,
        heartbeat: Heartbeat::new(),
        tokens: Tokens::new(),
    }
    .run()
    .await;
}

//joins the lobby,
// returns whether the lobby accepted
async fn join_lobby(
    socket: &mut WebSocket,
    user: &UserId,
    lobby: &mpsc::Sender<InnerCommand>,
    msg_sender: ClientMsgSender,
) -> bool {
    let (client_accepted_sender, client_accepted_receiver) = oneshot::channel();
    let connect = InnerCommand::Connect {
        user: user.clone(),
        msg_sender,
        client_accepted_sender,
    };
    if lobby.send(connect).await.is_err() {
        return false; // lobby is dead
    }

    if client_accepted_receiver.await != Ok(true) {
        let close = Message::Close(Some(CloseFrame {
            code: ERR_ALREADY_CONNECTED,
            reason: "user already connected".into(),
        }));
        send_websocket_msg(socket, close).await;
        return false; // the lobby refused
    }
    true
}

/// a client accepted by the lobby, owned by its task
struct Client {
    socket: WebSocket,
    user: UserId,
    lobby: mpsc::Sender<InnerCommand>,
    msg_sender: ClientMsgSender,
    msg_receiver: mpsc::Receiver<ServerMsg>,
    heartbeat: Heartbeat,
    tokens: Tokens,
}

impl Client {
    /// the client loop, notifies the lobby when the client leaves
    async fn run(mut self) {
        loop {
            let flow = tokio::select! {
                // the client sent us a message
                incoming = self.socket.recv() => self.on_incoming(incoming).await,

                // check that the client is still alive and ping it
                _ = self.heartbeat.wait_tick() => self.on_heartbeat_tick().await,

                // checks if the client is not slow to read messages
                _ = self.msg_sender.overflowed() => self.on_overflow(),

                // a part of the server wants to send a message to a client
                outgoing = self.msg_receiver.recv() => self.on_outgoing(outgoing).await,
            };
            if flow.is_break() {
                break;
            }
        }
        let _ = self
            .lobby
            .send(InnerCommand::Disconnect { user: self.user })
            .await;
    }

    async fn on_incoming(
        &mut self,
        incoming: Option<Result<Message, axum::Error>>,
    ) -> ControlFlow<()> {
        let Some(Ok(client_msg)) = incoming else {
            return ControlFlow::Break(());
        };
        self.heartbeat.seen();
        match client_msg {
            Message::Text(text) => self.on_text(text.as_str()).await,
            Message::Close(_) => ControlFlow::Break(()),
            _ => ControlFlow::Continue(()),
        }
    }

    async fn on_text(&mut self, text: &str) -> ControlFlow<()> {
        let msg = match serde_json::from_str::<ClientMsg>(text) {
            Ok(msg) => msg,
            Err(e) => {
                warn!(user = %self.user, %e, "invalid command");
                self.msg_sender
                    .send_error(format!("invalid command: {text}"));
                return ControlFlow::Continue(());
            }
        };
        if !self.tokens.try_consume_token(&msg) {
            self.msg_sender.send_error("too many messages, slow down");
            return ControlFlow::Continue(());
        }
        let cmd = InnerCommand::FromClient {
            from: self.user.clone(),
            msg,
        };
        if self.lobby.send(cmd).await.is_err() {
            return ControlFlow::Break(()); // lobby is dead
        }
        ControlFlow::Continue(())
    }

    async fn on_heartbeat_tick(&mut self) -> ControlFlow<()> {
        if !self.heartbeat.is_alive() {
            warn!(user = %self.user, "heartbeat timeout");
            return ControlFlow::Break(());
        }
        self.send_to_socket(Message::Ping(Bytes::new())).await
    }

    fn on_overflow(&self) -> ControlFlow<()> {
        warn!(user = %self.user, "exiting because client is too slow");
        ControlFlow::Break(())
    }

    async fn on_outgoing(&mut self, outgoing: Option<ServerMsg>) -> ControlFlow<()> {
        let Some(msg) = outgoing else {
            return ControlFlow::Break(());
        };
        let json = serde_json::to_string(&msg).expect("ServerMsg is always serializable");
        self.send_to_socket(Message::Text(json.into())).await
    }

    /// stops the client if the socket is closed or blocked for too long
    async fn send_to_socket(&mut self, msg: Message) -> ControlFlow<()> {
        if send_websocket_msg(&mut self.socket, msg).await {
            ControlFlow::Continue(())
        } else {
            ControlFlow::Break(())
        }
    }
}

/// sends a websocket message, returns false if the socket is closed or blocked for too long
async fn send_websocket_msg(socket: &mut WebSocket, msg: Message) -> bool {
    let result = timeout(CLIENT_SEND_TIMEOUT, socket.send(msg)).await;
    matches!(result, Ok(Ok(())))
}
