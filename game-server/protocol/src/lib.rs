use serde::{Deserialize, Serialize};
/// chat msg sent by clients
#[derive(Deserialize, Debug)]
#[serde(tag = "type", content = "payload", deny_unknown_fields)]
pub enum ClientMsg {
    Chat(ChatMsg),
    Matchmaking(MatchmakingMsg),
    // Game(GameMsg),
}

/// chat msg sent to clients
#[derive(Serialize, Debug)]
#[serde(tag = "type", content = "payload")]
pub enum ServerMsg {
    Chat(ChatEvent),
    Matchmaking(MatchmakingEvent),
    Error { message: String },
}

#[derive(Deserialize, Debug)]
#[serde(tag = "type", content = "payload", deny_unknown_fields)]
pub enum ChatMsg {
    Global { text: String },
    Private { to: String, text: String },
}

#[derive(Serialize, Debug)]
#[serde(tag = "type", content = "payload")]
pub enum ChatEvent {
    Global { from: String, text: String },
    Private { from: String, text: String },
}

#[derive(Deserialize, Debug, Clone, Copy)]
pub enum TeamMode {
    Solo,
    Duo,
}

impl TeamMode {
    /// number of players in each team
    pub fn team_size(self) -> usize {
        match self {
            TeamMode::Solo => 1,
            TeamMode::Duo => 2,
        }
    }
}

#[derive(Deserialize, Debug)]
#[serde(tag = "type", content = "payload", deny_unknown_fields)]
pub enum MatchmakingMsg {
    JoinQueue { team_mode: TeamMode },
    LeaveQueue {},
}

#[derive(Serialize, Debug)]
#[serde(tag = "type", content = "payload")]
pub enum MatchmakingEvent {
    Joined {},
    Left {},
    MatchFound {
        teammates: Vec<String>,
        opponents: Vec<String>,
    },
}
