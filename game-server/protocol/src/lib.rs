use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", content = "payload", deny_unknown_fields)]
pub enum ClientMsg {
    Chat(ChatMsg),
    // Matchmaking(MatchmakingMsg),
    // Game(GameMsg),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", content = "payload", deny_unknown_fields)]
pub enum ServerMsg {
    Chat(ChatEvent),
    Error { message: String },
}

/// chat msg sent by clients
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", content = "payload", deny_unknown_fields)]
pub enum ChatMsg {
    Global { text: String },
    Private { to: String, text: String },
}

/// chat msg sent to clients
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", content = "payload", deny_unknown_fields)]
pub enum ChatEvent {
    Global { from: String, text: String },
    Private { from: String, text: String },
}
