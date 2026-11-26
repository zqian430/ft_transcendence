//! all connected players are stored here, even the ones in a game

mod chat;
mod matchmaking;

use std::collections::{HashMap, hash_map::Entry};

use game_protocol::ClientMsg;
use tokio::{
    sync::{mpsc, oneshot},
    task::JoinSet,
};
use tracing::{error, info};

use crate::{
    client::ClientMsgSender,
    game::{self, GameId, GameIdCounter},
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

/// the lobby loop
pub async fn run(mut msg_receiver: mpsc::Receiver<InnerCommand>) {
    let mut players = Players::new();
    let mut matchmaking = Matchmaking::new();
    let mut in_game = InGame::new();
    let mut game_ids = GameIdCounter::new();
    let mut running_games = JoinSet::new();

    loop {
        tokio::select! {
            // a client sent a command
            cmd = msg_receiver.recv() => {
                let Some(cmd) = cmd else { break };
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
                                let game_id = game_ids.next_id();
                                start_game(&mut in_game, &mut running_games, game_id, match_info);
                            }
                        }
                    },
                }
            }

            // a game task is finished
            Some(finished) = running_games.join_next() => match finished {
                Ok(game_id) => {
                    info!("game {game_id} ended");
                    in_game.retain(|_, id| *id != game_id);
                }
                Err(err) => error!("a game crashed, its players are stuck in game: {err}"),
            }
        }
    }
}

fn start_game(
    in_game: &mut InGame,
    running_games: &mut JoinSet<GameId>,
    game_id: GameId,
    match_info: MatchInfo,
) {
    //flatten: [[a][b][c][d][e]] to [abcde]
    for user in match_info.teams.iter().flatten() {
        in_game.insert(user.clone(), game_id);
    }
    running_games.spawn(game::run(game_id, match_info.team_mode, match_info.teams));
}
