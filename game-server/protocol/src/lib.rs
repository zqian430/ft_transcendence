use serde::{Deserialize, Serialize};
/// chat msg sent by clients
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", content = "payload", deny_unknown_fields)]
pub enum ClientMsg {
    Chat(ChatMsg),
    Matchmaking(MatchmakingMsg),
    // Game(GameMsg),
}

/// chat msg sent to clients
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", content = "payload", deny_unknown_fields)]
pub enum ServerMsg {
    Chat(ChatEvent),
    Matchmaking(MatchmakingEvent),
    Error { message: String },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", content = "payload", deny_unknown_fields)]
pub enum ChatMsg {
    Global { text: String },
    Private { to: String, text: String },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", content = "payload", deny_unknown_fields)]
pub enum ChatEvent {
    Global { from: String, text: String },
    Private { from: String, text: String },
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TeamMode {
    Solo,
    Duo,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", content = "payload", deny_unknown_fields)]
pub enum MatchmakingMsg {
    JoinQueue { team_mode: TeamMode },
    LeaveQueue {},
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", content = "payload", deny_unknown_fields)]
pub enum MatchmakingEvent {
    Joined {},
    Left {},
    MatchFound {
        teammates: Vec<String>,
        opponents: Vec<String>,
    },
}
