use std::sync::Arc;

use game_protocol::ServerMsg;
use tokio::sync::{Notify, mpsc, mpsc::error::TrySendError};

use crate::constants::CLIENT_RESERVED_SLOTS;

#[derive(Clone)]
pub struct ClientMsgSender {
    sender: mpsc::Sender<ServerMsg>,
    overflow: Arc<Notify>,
}

impl ClientMsgSender {
    pub fn new(sender: mpsc::Sender<ServerMsg>) -> Self {
        Self {
            sender,
            overflow: Arc::new(Notify::new()),
        }
    }

    /// does not wait to not penalize everyone.
    /// if the client's queue is full it is too slow and gets disconnected
    pub fn send(&self, msg: ServerMsg) {
        if let Err(TrySendError::Full(_)) = self.sender.try_send(msg) {
            //notify that the client is too slow
            self.overflow.notify_one();
        }
    }

    /// sends a message. to use with messages that can be lost (ex chat)
    pub fn send_lossy(&self, msg: ServerMsg) {
        if self.sender.capacity() > CLIENT_RESERVED_SLOTS {
            let _ = self.sender.try_send(msg);
        }
    }

    ///finishes if the try_send failed (because the client is too slow to read)
    pub async fn overflowed(&self) {
        self.overflow.notified().await;
    }

    pub fn send_error(&self, message: impl Into<String>) {
        self.send(ServerMsg::Error {
            message: message.into(),
        });
    }
}
