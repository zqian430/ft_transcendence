use game_protocol::ServerMsg;
use tokio::sync::mpsc;

/// channel to a connected client, usable anywhere in the server (lobby, games, ...)
#[derive(Clone)]
pub struct ClientMsgSender(mpsc::Sender<ServerMsg>);

impl ClientMsgSender {
    pub fn new(sender: mpsc::Sender<ServerMsg>) -> Self {
        Self(sender)
    }

    /// never waits: a slow client must not block the caller
    /// the message is dropped if the client channel is full or closed
    pub fn send(&self, msg: ServerMsg) {
        let _ = self.0.try_send(msg);
    }

    pub fn send_error(&self, message: impl Into<String>) {
        self.send(ServerMsg::Error {
            message: message.into(),
        });
    }

    /// true if the client task has stopped
    pub fn is_closed(&self) -> bool {
        self.0.is_closed()
    }
}
