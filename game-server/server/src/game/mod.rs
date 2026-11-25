//! runs the game using the engine

use std::fmt;

/// only used within the server, postgre sql will generate a global id when the game finishes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GameId(u64);

impl fmt::Display for GameId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Game Id generator
#[derive(Default)]
pub struct GameIdCounter {
    next: u64,
}

impl GameIdCounter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn next_id(&mut self) -> GameId {
        let id = GameId(self.next);
        self.next += 1;
        id
    }
}
