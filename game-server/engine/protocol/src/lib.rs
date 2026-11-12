use serde::{Deserialize, Serialize};
// all message types sent by the server and the client are defined here

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", content = "payload")]

pub enum ClientMsg {
    Chat { text: String },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", content = "payload")]

pub enum ServerMsg {
    Chat { from: String, text: String },
}

#[cfg(test)]
mod tests {}
