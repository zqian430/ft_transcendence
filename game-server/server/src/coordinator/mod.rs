mod chat;

use std::collections::{HashMap, hash_map::Entry};

use game_protocol::{ClientMsg, ServerMsg};
use tokio::sync::{mpsc, oneshot};
use tracing::info;

pub type UserId = String;

pub type Players = HashMap<UserId, mpsc::Sender<ServerMsg>>;

/// inner protocol
pub enum InnerCommand {
    Connect {
        user: UserId,
        msg_sender: mpsc::Sender<ServerMsg>,
        client_accepted_sender: oneshot::Sender<bool>,
    },
    Disconnect {
        user: UserId,
    },

    /// a message sent by a client, forwarded as is
    FromClient {
        from: UserId,
        msg: ClientMsg,
    },
}

pub async fn run(mut msg_receiver: mpsc::Receiver<InnerCommand>) {
    let mut players = Players::new();

    while let Some(cmd) = msg_receiver.recv().await {
        match cmd {
            InnerCommand::Connect {
                user,
                msg_sender,
                client_accepted_sender,
            } => {
                let inserted = match players.entry(user.clone()) {
                    Entry::Vacant(e) => {
                        e.insert(msg_sender);
                        info!("player {user} connected");
                        true
                    }
                    Entry::Occupied(_) => {
                        info!("connection to user {user} refused: player already connected");
                        false
                    }
                };
                let _ = client_accepted_sender.send(inserted);
            }
            InnerCommand::Disconnect { user } => {
                players.remove(&user);
                info!("player {user} disconnected");
            }
            // commands sent by clients
            InnerCommand::FromClient { from, msg } => match msg {
                ClientMsg::Chat(msg) => chat::handle(&players, from, msg),
                // ClientMsg::Lobby(msg) => lobby::handle(&mut lobby, &players, from, msg),
            },
        }
    }
}
