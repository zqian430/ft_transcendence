use std::collections::{HashMap, hash_map::Entry};

use game_protocol::ServerMsg;
use tokio::sync::{mpsc, oneshot};
use tracing::info;

pub type UserId = String;

/// inner protocol, not sent by clients
pub enum InnerCommand {
    Connect {
        user: UserId,
        msg_sender: mpsc::Sender<ServerMsg>,
        client_accepted_sender: oneshot::Sender<bool>,
    },
    Disconnect {
        user: UserId,
    },
    ChatGlobal {
        from: UserId,
        text: String,
    },
    Chat {
        from: UserId,
        to: UserId,
        text: String,
    },
}

pub async fn run(mut msg_receiver: mpsc::Receiver<InnerCommand>) {
    let mut players: HashMap<UserId, mpsc::Sender<ServerMsg>> = HashMap::new();

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
            InnerCommand::ChatGlobal { from, text } => {
                for (player_name, msg_sender) in players.iter() {
                    if *player_name != from {
                        // try send instead of .await here, it blocks everyone if one client is too slow
                        let _ = msg_sender.try_send(ServerMsg::ChatGlobal {
                            from: from.clone(),
                            text: text.clone(),
                        });
                    }
                }
            }
            InnerCommand::Chat { from, to, text } => {
                if let Some(msg_sender_to_player) = players.get(&to) {
                    let _ = msg_sender_to_player.try_send(ServerMsg::Chat {
                        from: from.clone(),
                        text: text.clone(),
                    });
                } else if let Some(msg_sender_from_player) = players.get(&from) {
                    let _ = msg_sender_from_player.try_send(ServerMsg::Error {
                        message: format!("user {to} is not connected"),
                    });
                }
            }
        }
    }
}
