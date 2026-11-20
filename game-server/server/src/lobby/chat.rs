use game_protocol::{ChatEvent, ChatMsg, ServerMsg};

use super::{Players, UserId};

pub fn handle(players: &Players, from: UserId, msg: ChatMsg) {
    match msg {
        ChatMsg::Global { text } => chat_global(players, from, text),
        ChatMsg::Private { to, text } => chat_private(players, from, to, text),
    }
}

fn chat_global(players: &Players, from: UserId, text: String) {
    for (player_name, msg_sender) in players.iter() {
        if *player_name != from {
            msg_sender.send(ServerMsg::Chat(ChatEvent::Global {
                from: from.clone(),
                text: text.clone(),
            }));
        }
    }
}

fn chat_private(players: &Players, from: UserId, to: UserId, text: String) {
    if let Some(msg_sender_to_player) = players.get(&to) {
        msg_sender_to_player.send(ServerMsg::Chat(ChatEvent::Private { from, text }));
    } else if let Some(msg_sender_from_player) = players.get(&from) {
        msg_sender_from_player.send_error(format!("user {to} is not connected"));
    }
}
