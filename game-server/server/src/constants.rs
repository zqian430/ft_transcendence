use std::time::Duration;

pub const SERVER_LISTEN_HOST: &str = "0.0.0.0";
pub const SERVER_PORT_ENV: &str = "GAME_SERVER_PORT";
pub const CLIENT_MAX_MSG_SIZE: usize = 1024 * 4;
pub const CLIENT_CHANNEL_SIZE: usize = 160;
pub const CLIENT_RESERVED_SLOTS: usize = 32;
pub const CHAT_MAX_LEN: usize = 200; // in nb of chars
pub const CHAT_MAX_TOKENS: f32 = 3.0; // only chat tokens
pub const GLOBAL_MAX_TOKENS: f32 = 30.0; // all commands tokens ( chat included)
pub const CHAT_TOKENS_PER_SEC: f32 = 1.0;
pub const GLOBAL_TOKENS_PER_SEC: f32 = 30.0;
pub const CLIENT_SEND_TIMEOUT: Duration = Duration::from_secs(5);
pub const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(3); // check every  HEARTBEAT_INTERVAL secs
pub const HEARTBEAT_TIMEOUT: Duration = Duration::from_secs(10);
pub const LOBBY_CHANNEL_SIZE: usize = 1024;
pub const GAME_TICK: Duration = Duration::from_millis(16); // ~60 ticks per sec
pub const GAME_TEMP_DURATION: Duration = Duration::from_secs(20); //temporary
pub const ERR_ALREADY_CONNECTED: u16 = 4001;
