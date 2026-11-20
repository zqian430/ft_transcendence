use game_protocol::ServerMsg;
use tokio::sync::mpsc;

#[derive(Clone)]
pub struct ClientMsgSender(mpsc::Sender<ServerMsg>);

impl ClientMsgSender {
    pub fn new(sender: mpsc::Sender<ServerMsg>) -> Self {
        Self(sender)
    }

    /// does not wait to not penalize everyone
    pub fn send(&self, msg: ServerMsg) {
        let _ = self.0.try_send(msg);
    }

    pub fn send_error(&self, message: impl Into<String>) {
        self.send(ServerMsg::Error {
            message: message.into(),
        });
    }
}
