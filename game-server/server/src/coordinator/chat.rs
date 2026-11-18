use game_protocol::{ChatEvent, ChatMsg, ServerMsg};

use super::{Players, UserId};

pub fn handle(players: &Players, from: UserId, msg: ChatMsg) {
    match msg {
        ChatMsg::Global { text } => global(players, from, text),
        ChatMsg::Private { to, text } => private(players, from, to, text),
    }
}

fn global(players: &Players, from: UserId, text: String) {
    for (player_name, msg_sender) in players.iter() {
        if *player_name != from {
            // try send instead of .await here, it blocks everyone if one client is too slow
            let _ = msg_sender.try_send(ServerMsg::Chat(ChatEvent::Global {
                from: from.clone(),
                text: text.clone(),
            }));
        }
    }
}

fn private(players: &Players, from: UserId, to: UserId, text: String) {
    if let Some(msg_sender_to_player) = players.get(&to) {
        let _ = msg_sender_to_player.try_send(ServerMsg::Chat(ChatEvent::Private { from, text }));
    } else if let Some(msg_sender_from_player) = players.get(&from) {
        let _ = msg_sender_from_player.try_send(ServerMsg::Error {
            message: format!("user {to} is not connected"),
        });
    }
}
