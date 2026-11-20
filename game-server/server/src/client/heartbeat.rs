use tokio::time::{Instant, Interval, MissedTickBehavior, interval_at};

use crate::constants::{HEARTBEAT_INTERVAL, HEARTBEAT_TIMEOUT};

/// detects dead connections that did not send a Close.
/// it sends a Ping every tick and browsers answer with a Pong
pub struct Heartbeat {
    interval: Interval,
    last_seen: Instant,
}

impl Heartbeat {
    pub fn new() -> Self {
        let now = Instant::now();
        let mut interval = interval_at(now + HEARTBEAT_INTERVAL, HEARTBEAT_INTERVAL);
        // if the server is slow just delay the next ping
        interval.set_missed_tick_behavior(MissedTickBehavior::Delay);
        Self {
            interval,
            last_seen: now,
        }
    }

    /// waits until it's time to check the client and send a new ping
    pub async fn wait_tick(&mut self) {
        self.interval.tick().await;
    }

    /// updates the last time a client sent a message
    pub fn seen(&mut self) {
        self.last_seen = Instant::now();
    }

    pub fn is_alive(&self) -> bool {
        self.last_seen.elapsed() < HEARTBEAT_TIMEOUT
    }
}
