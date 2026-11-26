//! runs the game using the engine

use std::fmt;

use game_protocol::TeamMode;
use tokio::time::{MissedTickBehavior, interval, sleep};
use tracing::info;

use crate::{
    constants::{GAME_TEMP_DURATION, GAME_TICK},
    lobby::UserId,
};

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

/// runs a game
pub async fn run(id: GameId, team_mode: TeamMode, teams: [Vec<UserId>; 2]) -> GameId {
    info!(
        "game {id} started in {team_mode:?}: {:?} vs {:?}",
        teams[0], teams[1]
    );

    let mut tick = interval(GAME_TICK);
    // skip missed ticks if the server is slow
    tick.set_missed_tick_behavior(MissedTickBehavior::Skip);

    // temporary
    let game_over = sleep(GAME_TEMP_DURATION);
    tokio::pin!(game_over);

    loop {
        tokio::select! {
            _ = tick.tick() => {

            }
            _ = &mut game_over => break,
        }
    }
    return id;
}
