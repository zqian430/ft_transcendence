// all message types sent by the server and the client are defined here
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", content = "payload")]
pub enum ClientMsg {
    Chat { text: String },
    ChatPrivate {from: String, to: String, text: String},
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", content = "payload")]
pub enum ServerMsg {
    Chat { from: String, text: String },
    ChatPrivate {from: String, text: String},
}
