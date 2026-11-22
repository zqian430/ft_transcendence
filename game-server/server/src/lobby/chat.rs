use game_protocol::{ChatEvent, ChatMsg, ServerMsg};

use crate::constants::CHAT_MAX_LEN;

use super::{Players, UserId};

pub fn handle(players: &Players, from: UserId, msg: ChatMsg) {
    match msg {
        ChatMsg::Global { text } => chat_global(players, from, text),
        ChatMsg::Private { to, text } => chat_private(players, from, to, text),
    }
}

fn chat_global(players: &Players, from: UserId, text: String) {
    if let Some(err_mes) = check_valid_chat_message(&text) {
        if let Some(msg_sender_from_player) = players.get(&from) {
            msg_sender_from_player.send_error(err_mes);
        }
        return;
    }
    for (player_name, msg_sender) in players.iter() {
        if *player_name != from {
            msg_sender.send_lossy(ServerMsg::Chat(ChatEvent::Global {
                from: from.clone(),
                text: text.clone(),
            }));
        }
    }
}

fn chat_private(players: &Players, from: UserId, to: UserId, text: String) {
    if let Some(err_mes) = check_valid_chat_message(&text) {
        if let Some(msg_sender_from_player) = players.get(&from) {
            msg_sender_from_player.send_error(err_mes);
        }
        return;
    }

    if let Some(msg_sender_to_player) = players.get(&to) {
        msg_sender_to_player.send_lossy(ServerMsg::Chat(ChatEvent::Private { from, text }));
    } else if let Some(msg_sender_from_player) = players.get(&from) {
        msg_sender_from_player.send_error(format!("user {to} is not connected"));
    }
}

fn check_valid_chat_message(text: &str) -> Option<&str> {
    if text.chars().count() > CHAT_MAX_LEN {
        return Some("message exceeds max length");
    }
    None
}
