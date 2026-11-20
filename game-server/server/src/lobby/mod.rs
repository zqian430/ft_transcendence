//! all connected players are stored here, even the ones in a game

mod chat;
mod matchmaking;

use std::collections::{HashMap, hash_map::Entry};

use game_protocol::ClientMsg;
use tokio::sync::{mpsc, oneshot};
use tracing::info;

use crate::{client_msg_sender::ClientMsgSender, lobby::matchmaking::Matchmaking};

pub type UserId = String;
type Players = HashMap<UserId, ClientMsgSender>;

/// all commands used in the server, not just the ones in the protocol
pub enum InnerCommand {
    Connect {
        user: UserId,
        msg_sender: ClientMsgSender,
        client_accepted_sender: oneshot::Sender<bool>,
    },
    Disconnect {
        user: UserId,
    },

    /// a message sent by a client (protocol commands)
    FromClient {
        from: UserId,
        msg: ClientMsg,
    },
}

pub async fn run(mut msg_receiver: mpsc::Receiver<InnerCommand>) {
    let mut players = Players::new();
    let mut matchmaking = Matchmaking::new();

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
                matchmaking.remove(&user);
                info!("player {user} disconnected");
            }
            InnerCommand::FromClient { from, msg } => match msg {
                ClientMsg::Chat(msg) => chat::handle(&players, from, msg),
                ClientMsg::Matchmaking(msg) => {
                    if let Some(_match_info) = matchmaking.handle(&players, from, msg) {
                        // then start match
                    }
                }
            },
        }
    }
}
