use std::collections::VecDeque;

use game_protocol::{MatchmakingEvent, MatchmakingMsg, ServerMsg, TeamMode};
use tracing::info;

use crate::lobby::{InGame, Players, UserId};

/// returned to the lobby when enough players were found to start a game
pub struct MatchInfo {
    pub team_mode: TeamMode,
    pub teams: [Vec<UserId>; 2],
}

#[derive(Default)]
pub struct Matchmaking {
    solo_queue: VecDeque<UserId>,
    duo_queue: VecDeque<UserId>,
}

impl Matchmaking {
    pub fn new() -> Self {
        Self::default()
    }

    /// handles a matchmaking command from a client
    pub fn handle(
        &mut self,
        players: &Players,
        in_game: &InGame,
        from: UserId,
        msg: MatchmakingMsg,
    ) -> Option<MatchInfo> {
        let msg_sender = players.get(&from)?;

        match msg {
            MatchmakingMsg::JoinQueue { team_mode } => {
                if let Some(game_id) = in_game.get(&from) {
                    msg_sender.send_error(format!("already in game {game_id}"));
                    return None;
                }
                if self.is_in_queue(&from) {
                    msg_sender.send_error("already in queue");
                    return None;
                }
                info!("player {from} joined the {team_mode:?} queue");
                self.queue_mut(team_mode).push_back(from);
                msg_sender.send(ServerMsg::Matchmaking(MatchmakingEvent::Joined {}));
                self.get_match(players, team_mode)
            }
            MatchmakingMsg::LeaveQueue {} => {
                if self.remove(&from) {
                    info!("player {from} left the queue");
                    msg_sender.send(ServerMsg::Matchmaking(MatchmakingEvent::Left {}));
                } else {
                    msg_sender.send_error("not in queue");
                }
                None
            }
        }
    }

    ///removes the user from all queues, returns whether the user was removed
    pub fn remove(&mut self, to_remove: &UserId) -> bool {
        let before = self.solo_queue.len() + self.duo_queue.len();
        self.solo_queue.retain(|user| user != to_remove);
        self.duo_queue.retain(|user| user != to_remove);
        before != self.solo_queue.len() + self.duo_queue.len()
    }

    fn is_in_queue(&self, user: &UserId) -> bool {
        self.solo_queue.contains(user) || self.duo_queue.contains(user)
    }

    fn queue_mut(&mut self, team_mode: TeamMode) -> &mut VecDeque<UserId> {
        match team_mode {
            TeamMode::Solo => &mut self.solo_queue,
            TeamMode::Duo => &mut self.duo_queue,
        }
    }

    /// takes the first players of the queue if there are enough to fill two teams
    fn get_match(&mut self, players: &Players, team_mode: TeamMode) -> Option<MatchInfo> {
        let team_size = team_mode.team_size();
        let queue = self.queue_mut(team_mode);

        if queue.len() < team_size * 2 {
            return None;
        }
        let team_a: Vec<UserId> = queue.drain(..team_size).collect();
        let team_b: Vec<UserId> = queue.drain(..team_size).collect();
        info!("match found between teams: {team_a:?} vs {team_b:?}");

        notify_team_match_found(players, &team_a, &team_b);
        notify_team_match_found(players, &team_b, &team_a);
        Some(MatchInfo {
            team_mode,
            teams: [team_a, team_b],
        })
    }
}

/// sends MatchFound to every player of the team
fn notify_team_match_found(players: &Players, team: &[UserId], opponents: &[UserId]) {
    for user in team {
        let Some(msg_sender) = players.get(user) else {
            continue;
        };
        let teammates = team.iter().filter(|u| *u != user).cloned().collect();
        msg_sender.send(ServerMsg::Matchmaking(MatchmakingEvent::MatchFound {
            teammates,
            opponents: opponents.to_vec(),
        }));
    }
}
