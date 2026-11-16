use std::collections::HashMap;

use game_protocol::ServerMsg;
use tokio::sync::mpsc;
use tracing::info;

pub type UserId = String;

/// inner protocol, not sent by clients
pub enum Command {
    Connect {
        user: UserId,
        msg_sender: mpsc::Sender<ServerMsg>,
    },
    Disconnect {
        user: UserId,
    },
    ChatGlobal {
        from: UserId,
        text: String,
    },
    ChatUser {
        from: UserId,
        to: UserId,
        text: String,
    },
}

pub async fn run(mut msg_receiver: mpsc::Receiver<Command>) {
    let mut players: HashMap<UserId, mpsc::Sender<ServerMsg>> = HashMap::new();

    while let Some(cmd) = msg_receiver.recv().await {
        match cmd {
            Command::Connect { user, msg_sender } => {
                players.insert(user.clone(), msg_sender);
                info!("player {user} connected");
            }
            Command::Disconnect { user } => {
                players.remove(&user);
                info!("player {user} disconnected");
            }
            Command::ChatGlobal { from, text } => {
                for msg_sender in players.values() {
                    // try send instead of .await here, it blocks everyone if one client is too slow
                    let _ = msg_sender.try_send(ServerMsg::Chat {
                        from: from.clone(),
                        text: text.clone(),
                    });
                }
            }
            Command::ChatUser { from, to, text } => {
                if let Some(msg_sender_to_player) = players.get(&to) {
                    let _ = msg_sender_to_player.try_send(ServerMsg::ChatPrivate {
                        from: from.clone(),
                        text: text.clone(),
                    });
                } else {
                    continue;
                }
            }
        }
    }
}
