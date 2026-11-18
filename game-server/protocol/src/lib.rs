// all message types sent by the server and the client are defined here
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", content = "payload", deny_unknown_fields)]
pub enum ClientMsg {
    ChatGlobal { text: String },
    Chat { to: String, text: String },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", content = "payload", deny_unknown_fields)]
pub enum ServerMsg {
    ChatGlobal { from: String, text: String },
    Chat { from: String, text: String },
    Error { message: String },
}
