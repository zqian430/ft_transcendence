//! all connected players are stored here, even the ones in a game

mod chat;
mod matchmaking;

use std::collections::{HashMap, hash_map::Entry};

use game_protocol::ClientMsg;
use tokio::{
    sync::{mpsc, oneshot},
    task::{JoinError, JoinSet},
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

/// state of the lobby, owned by its task
pub struct Lobby {
    msg_receiver: mpsc::Receiver<InnerCommand>,
    players: Players,
    matchmaking: Matchmaking,
    in_game: InGame,
    game_ids: GameIdCounter,
    running_games: JoinSet<GameId>,
}

impl Lobby {
    pub fn new(msg_receiver: mpsc::Receiver<InnerCommand>) -> Self {
        Self {
            msg_receiver,
            players: Players::new(),
            matchmaking: Matchmaking::new(),
            in_game: InGame::new(),
            game_ids: GameIdCounter::new(),
            running_games: JoinSet::new(),
        }
    }

    /// the lobby loop
    pub async fn run(mut self) {
        loop {
            tokio::select! {
                // a client sent a command
                cmd = self.msg_receiver.recv() => {
                    let Some(cmd) = cmd else { break };
                    self.on_command(cmd);
                }

                // a game task is finished
                Some(finished) = self.running_games.join_next() => self.on_game_finished(finished),
            }
        }
    }

    fn on_command(&mut self, cmd: InnerCommand) {
        match cmd {
            InnerCommand::Connect {
                user,
                msg_sender,
                client_accepted_sender,
            } => {
                let accepted = self.on_connect(user, msg_sender);
                let _ = client_accepted_sender.send(accepted);
            }
            InnerCommand::Disconnect { user } => self.on_disconnect(user),
            InnerCommand::FromClient { from, msg } => self.on_client_msg(from, msg),
        }
    }

    /// returns whether the client is accepted
    fn on_connect(&mut self, user: UserId, msg_sender: ClientMsgSender) -> bool {
        match self.players.entry(user) {
            Entry::Vacant(entry) => {
                info!("player {} connected", entry.key());
                entry.insert(msg_sender);
                true
            }
            Entry::Occupied(entry) => {
                info!(
                    "connection to user {} refused: player already connected",
                    entry.key()
                );
                false
            }
        }
    }

    fn on_disconnect(&mut self, user: UserId) {
        self.players.remove(&user);
        self.matchmaking.remove(&user);
        info!("player {user} disconnected");
    }

    fn on_client_msg(&mut self, from: UserId, msg: ClientMsg) {
        match msg {
            ClientMsg::Chat(msg) => chat::handle(&self.players, from, msg),
            ClientMsg::Matchmaking(msg) => {
                if let Some(match_info) =
                    self.matchmaking
                        .handle(&self.players, &self.in_game, from, msg)
                {
                    self.start_game(match_info);
                }
            }
        }
    }

    fn on_game_finished(&mut self, finished: Result<GameId, JoinError>) {
        match finished {
            Ok(game_id) => {
                info!("game {game_id} ended");
                self.in_game.retain(|_, id| *id != game_id);
            }
            Err(err) => error!("a game crashed, its players are stuck in game: {err}"),
        }
    }

    fn start_game(&mut self, match_info: MatchInfo) {
        let game_id = self.game_ids.next_id();
        //flatten: [[a][b][c][d][e]] to [abcde]
        for user in match_info.teams.iter().flatten() {
            self.in_game.insert(user.clone(), game_id);
        }
        self.running_games
            .spawn(game::run(game_id, match_info.team_mode, match_info.teams));
    }
}
