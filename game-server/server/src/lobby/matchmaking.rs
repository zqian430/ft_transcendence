use std::collections::VecDeque;

use game_protocol::{MatchmakingEvent, MatchmakingMsg, ServerMsg, TeamMode};
use tracing::info;

use crate::lobby::{Players, UserId};

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
        from: UserId,
        msg: MatchmakingMsg,
    ) -> Option<MatchInfo> {
        let msg_sender = players.get(&from)?;

        match msg {
            MatchmakingMsg::JoinQueue { team_mode } => {
                if self.is_queued(&from) {
                    msg_sender.send_error("already in queue");
                    return None;
                }
                info!("player {from} joined the {team_mode:?} queue");
                self.queue_mut(team_mode).push_back(from);
                msg_sender.send(ServerMsg::Matchmaking(MatchmakingEvent::Joined {}));
                self.try_match(players, team_mode)
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

    fn is_queued(&self, user: &UserId) -> bool {
        self.solo_queue.contains(user) || self.duo_queue.contains(user)
    }

    fn queue_mut(&mut self, team_mode: TeamMode) -> &mut VecDeque<UserId> {
        match team_mode {
            TeamMode::Solo => &mut self.solo_queue,
            TeamMode::Duo => &mut self.duo_queue,
        }
    }

    /// takes the first players of the queue if there are enough to fill two teams
    fn try_match(&mut self, players: &Players, team_mode: TeamMode) -> Option<MatchInfo> {
        let team_size = players_per_team(team_mode);
        let queue = self.queue_mut(team_mode);

        // a client can be dead while its Disconnect is still waiting in the lobby channel
        queue.retain(|u| players.get(u).is_some_and(|sender| !sender.is_closed()));

        if queue.len() < team_size * 2 {
            return None;
        }
        let team_a: Vec<UserId> = queue.drain(..team_size).collect();
        let team_b: Vec<UserId> = queue.drain(..team_size).collect();
        info!("{team_mode:?} match found: {team_a:?} vs {team_b:?}");

        notify_team(players, &team_a, &team_b);
        notify_team(players, &team_b, &team_a);
        Some(MatchInfo {
            team_mode,
            teams: [team_a, team_b],
        })
    }
}

fn players_per_team(team_mode: TeamMode) -> usize {
    match team_mode {
        TeamMode::Solo => 1,
        TeamMode::Duo => 2,
    }
}

/// sends MatchFound to every player of `team`
fn notify_team(players: &Players, team: &[UserId], opponents: &[UserId]) {
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
