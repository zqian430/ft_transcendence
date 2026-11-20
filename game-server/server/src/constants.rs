use std::time::Duration;

pub const SERVER_LISTEN_HOST: &str = "0.0.0.0";
pub const SERVER_PORT_ENV: &str = "GAME_SERVER_PORT";
pub const CLIENT_MAX_MSG_SIZE: usize = 1024 * 16;
pub const CLIENT_CHANNEL_SIZE: usize = 128;
pub const CLIENT_SEND_TIMEOUT: Duration = Duration::from_secs(5);
pub const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5); // check every  HEARTBEAT_INTERVAL secs
pub const HEARTBEAT_TIMEOUT: Duration = Duration::from_secs(15);
pub const LOBBY_CHANNEL_SIZE: usize = 1024;
pub const ERR_ALREADY_CONNECTED: u16 = 4001;
