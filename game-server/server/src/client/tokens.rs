use game_protocol::ClientMsg;
use tokio::time::Instant;

use crate::constants::{
    CHAT_MAX_TOKENS, CHAT_TOKENS_PER_SEC, GLOBAL_MAX_TOKENS, GLOBAL_TOKENS_PER_SEC,
};

/// limits how many commands a client can send
pub struct Tokens {
    chat_tokens: f32,
    global_tokens: f32,
    last_refill: Instant,
}

impl Tokens {
    pub fn new() -> Self {
        Self {
            chat_tokens: 0.0, // 0 because else players could spam chat by reconnecting
            global_tokens: GLOBAL_MAX_TOKENS,
            last_refill: Instant::now(),
        }
    }

    /// returns whether there is available tokens for this msg type
    pub fn try_consume_token(&mut self, msg: &ClientMsg) -> bool {
        self.refill();
        match msg {
            ClientMsg::Chat(_) => {
                if self.chat_tokens < 1.0 || !self.try_consume_global() {
                    return false;
                }
                self.chat_tokens -= 1.0;
                true
            }
            _ => self.try_consume_global(),
        }
    }

    pub fn try_consume_global(&mut self) -> bool {
        self.refill();
        if self.global_tokens < 1.0 {
            return false;
        }
        self.global_tokens -= 1.0;
        true
    }

    fn refill(&mut self) {
        let now = Instant::now();
        let secs = now.duration_since(self.last_refill).as_secs_f32();
        self.chat_tokens = (self.chat_tokens + secs * CHAT_TOKENS_PER_SEC).min(CHAT_MAX_TOKENS);
        self.global_tokens =
            (self.global_tokens + secs * GLOBAL_TOKENS_PER_SEC).min(GLOBAL_MAX_TOKENS);
        self.last_refill = now;
    }
}
