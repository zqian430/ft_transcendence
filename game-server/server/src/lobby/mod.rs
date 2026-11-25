//! all connected players are stored here, even the ones in a game

mod chat;
mod matchmaking;

use std::collections::{HashMap, hash_map::Entry};

use game_protocol::ClientMsg;
use tokio::sync::{mpsc, oneshot};
use tracing::info;

use crate::{
    client::ClientMsgSender,
    game::{GameId, GameIdCounter},
    lobby::matchmaking::{MatchInfo, Matchmaking},
};

pub type UserId = String;
type Players = HashMap<UserId, ClientMsgSender>;
type InGame = HashMap<UserId, GameId>;

/// all commands used in the server, not just the ones in the protocol
pub enum InnerCommand {
    Connect {
        user: UserId,
        msg_sender: ClientMsgSender,
        client_accepted_sender: oneshot::Sender<bool>,
    },
    Disconnect {
        user: UserId,
    },

    /// a message sent by a client (protocol commands)
    FromClient {
        from: UserId,
        msg: ClientMsg,
    },
}

pub async fn run(mut msg_receiver: mpsc::Receiver<InnerCommand>) {
    let mut players = Players::new();
    let mut matchmaking = Matchmaking::new();
    let mut in_game = InGame::new();
    let mut game_ids = GameIdCounter::new();

    while let Some(cmd) = msg_receiver.recv().await {
        match cmd {
            InnerCommand::Connect {
                user,
                msg_sender,
                client_accepted_sender,
            } => {
                let inserted = match players.entry(user.clone()) {
                    Entry::Vacant(entry) => {
                        entry.insert(msg_sender);
                        info!("player {user} connected");
                        true
                    }
                    Entry::Occupied(_) => {
                        info!("connection to user {user} refused: player already connected");
                        false
                    }
                };
                let _ = client_accepted_sender.send(inserted);
            }
            InnerCommand::Disconnect { user } => {
                players.remove(&user);
                matchmaking.remove(&user);
                info!("player {user} disconnected");
            }
            InnerCommand::FromClient { from, msg } => match msg {
                ClientMsg::Chat(msg) => chat::handle(&players, from, msg),
                ClientMsg::Matchmaking(msg) => {
                    if let Some(match_info) = matchmaking.handle(&players, &in_game, from, msg) {
                        start_game(&mut in_game, &mut game_ids, match_info);
                    }
                }
            },
        }
    }
}

/// registers all the players of the match as in game
fn start_game(in_game: &mut InGame, game_ids: &mut GameIdCounter, match_info: MatchInfo) {
    let game_id = game_ids.next_id();
    //flatten: [[a][b][c][d][e]] to [abcde]
    for user in match_info.teams.iter().flatten() {
        in_game.insert(user.clone(), game_id);
    }
    info!(
        "game {game_id} started in {:?}: {:?} vs {:?}",
        match_info.team_mode, match_info.teams[0], match_info.teams[1]
    );
    // TODO: spawn the game task
}
