use std::{ collections::VecDeque, mem};
use std::time::{Instant, Duration};

pub mod card;
pub mod player;
pub mod duel;

pub use crate::constant::ALL_PLAYER_ID;

pub use card::{Card};
pub use player::{
    Player,
    PlayStrategy,
    PlayStrategy::RandomStrategy,
    PlayStrategy::BalancedStrategy,
    PlayStrategy::ProbabilityStrategy,
    PlayStrategy::MetaRandomStrategy,
    PlayStrategy::OnlineStrategy,
};
use duel::{Duel, DuelStatus};

pub enum Availability<T> {
    Available(T),
    Waiting,
}

struct RestingPlayer {
    player: Player,
    ready_at: Instant,
}

enum GameStatus {
    Waiting,
    Playing,
    Finished(Instant) // timeout
}

pub struct Game {
    players: Vec<Player>,
    status: GameStatus,
    playable_card_count: CardCount,
    resting_players: Vec<RestingPlayer>, // Idle -> Waiting -> Duel -> Idle
    waiting_players: VecDeque<Player>, // Waiting for duel
    ongoing_duels: Vec<Duel>,
    winners: Vec<PlayerInfo>,
    losers: Vec<PlayerInfo>,
    player_logs: Vec<PlayerLog>,

    target_star: usize,
    duel_duration: Duration,
    rest_duration: Duration,
    restart_duration: Duration,
}

impl Default for Game {
    fn default() -> Self {
        Self::new()
    }
}

impl Game {
    pub fn new () -> Game {
        Game{
            players: Vec::new(),
            status: GameStatus::Waiting,
            playable_card_count: CardCount::new(0),
            resting_players: Vec::new(),
            waiting_players: VecDeque::new(),
            ongoing_duels: Vec::new(),
            winners: Vec::new(),
            losers: Vec::new(),
            player_logs: Vec::new(),

            target_star: 5,
            duel_duration: Duration::from_secs(20),
            rest_duration: Duration::from_secs(10),
            restart_duration: Duration::from_secs(20),
        }
    }
    pub fn add_players(&mut self, mut players: Vec<Player>) {
        if let GameStatus::Waiting = self.status {
            self.players.append(&mut players);
        }

    }
    pub fn add_player(&mut self, player: Player) {
        if let GameStatus::Waiting = self.status {
            self.players.push(player);
        }
    }

    pub fn remove_player(&mut self, player_id: usize) {
        if let GameStatus::Waiting = self.status {
            self.players.retain(|player| {player.id != player_id})
        }
    }


    fn add_bot(&mut self, bot_id: &mut usize) {
        *bot_id -= 7;

        let strategies: [fn() -> PlayStrategy; 4] = [
            || RandomStrategy,
            || BalancedStrategy,
            || ProbabilityStrategy,
            PlayStrategy::meta_random,
        ];

        let strategy = strategies[rand::random_range(0..strategies.len())];
        let player = Player::new(
            *bot_id,
            format!("Bot-{}", *bot_id),
            strategy(),
        );

        self.add_player(player);
    }

    pub fn add_bots(&mut self, count: usize) {
        if let GameStatus::Waiting = self.status {
            let mut bot_id = 1000;

            for _ in 0..count {
                self.add_bot(&mut bot_id);
            }
        }
    }

    pub fn fill_with_bots(&mut self, max_player: usize) {
        if let GameStatus::Waiting = self.status {
            let mut bot_id = 1000;

            while self.players.len() < max_player {
                self.add_bot(&mut bot_id);
            }
        }
    }

    pub fn finish_resting(&mut self, player_id: usize) {
        if let Some(index) = self.resting_players.iter()
            .position(|x| x.player.id == player_id) {
            let resting_player = self.resting_players.remove(index);
            self.waiting_players.push_back(resting_player.player);
        }
    }

    pub fn retain_players(&mut self, ids: &[usize]) {
        if let GameStatus::Waiting = self.status {
            self.players.retain(|player| ids.contains(&player.id));
        }
    }

    pub fn start(&mut self) {
        let initial_star = 3;
        let total_player_card_per_type = 4;
        let total_player = self.players.len();
        
        for player in &mut self.players {
            let mut initial_cards = Vec::new();
            for _ in 0..total_player_card_per_type {
                initial_cards.push(Card::Rock);
                initial_cards.push(Card::Paper);
                initial_cards.push(Card::Scissors);
            }

            player.init(initial_star, initial_cards);
        }
        
        let total_card_per_type = total_player_card_per_type * total_player;
        self.playable_card_count.reset(total_card_per_type);
        self.resting_players = Vec::new();
        self.ongoing_duels = Vec::new();
        self.winners = Vec::new();
        self.losers = Vec::new();
        self.player_logs = Vec::new();

        self.waiting_players = mem::take(&mut self.players).into();
        self.status = GameStatus::Playing;
    }

    pub fn update(&mut self) -> (GameState, Vec<PlayerUpdate>, Vec<PlayerLog>) {
        match self.status {
            GameStatus::Waiting => {
                let (player_infos, player_updates)
                    : (Vec<PlayerInfo>, Vec<PlayerUpdate>) = self.players.iter()
                    .map(|x| {
                        (
                            PlayerInfo {
                                id: x.id,
                                name: x.name.clone(),
                                status: 0,
                            },
                            PlayerUpdate {
                                player_id: x.id,
                                state: PlayerState::Joined,
                            }
                        )
                    })
                    .unzip();

                let game_state = GameState::Waiting { player_infos };
                let player_logs = std::mem::take(&mut self.player_logs);
                (game_state, player_updates, player_logs)
            },

            GameStatus::Playing => {
                if self.playable_card_count.total() == 0 {
                    let restart_at = Instant::now() + self.restart_duration;
                    self.status = GameStatus::Finished(restart_at);

                    let game_state = GameState::Finished {
                        winners: self.winners.clone(),
                        remaining_secs: restart_at
                            .saturating_duration_since(Instant::now())
                            .as_secs(),
                    };
                    let player_updates = self.get_in_game_player_updates();
                    let player_logs = std::mem::take(&mut self.player_logs);
                    return (game_state, player_updates, player_logs);
                } 
                let mut players_still_resting = Vec::new();
                for resting_player in self.resting_players.drain(..){
                    if Instant::now() > resting_player.ready_at {
                        self.waiting_players.push_back(resting_player.player);
                    } else {
                        players_still_resting.push(resting_player);
                    }
                }
                self.resting_players = players_still_resting;

                let mut remaining_duels = Vec::new();
                for duel in self.ongoing_duels.drain(..) {
                    match duel.play(&self.playable_card_count) {
                        DuelStatus::Waiting(duel) => remaining_duels.push(duel),
                        DuelStatus::Finished(duel_result) => {
                            let ready_at = Instant::now() + self.rest_duration;
                            let description = 
                                    "Duel cancelled: player was not ready";
                            if duel_result.returned_cards.is_empty() {
                                // Players not ready
                                let player1 = duel_result.player1;
                                let player2 = duel_result.player2;

                                self.player_logs.push( PlayerLog {
                                    player_id: player1.id,
                                    message: description.to_string()
                                });
                                self.resting_players.push( RestingPlayer {
                                    player: player1,
                                    ready_at,
                                });

                                self.player_logs.push( PlayerLog {
                                    player_id: player2.id,
                                    message: description.to_string()
                                });
                                self.resting_players.push( RestingPlayer {
                                    player: player2,
                                    ready_at,
                                });
                            } else {
                                self.playable_card_count
                                    .remove(duel_result.returned_cards);

                                // Return player
                                let player1 = duel_result.player1;
                                let player2 = duel_result.player2;

                                let player1_name = player1.name.clone();
                                let player2_name = player2.name.clone();

                                let player1_result = duel_result.player1_result;
                                let player2_result = duel_result.player2_result;

                                if let (
                                    Availability::Available(player1_card),
                                    Availability::Available(player2_card)
                                ) = (duel_result.player1_card, duel_result.player2_card) 
                                {
                                    // Player 1
                                    if !player1.cards.is_empty() && player1.star > 0 {
                                        self.player_logs.push( PlayerLog { 
                                            player_id: player1.id,
                                            message:format!("{} against {} — {} vs {}",
                                                player1_result.description(),
                                                player2_name,
                                                player1_card.emoji(),
                                                player2_card.emoji(),
                                            ),
                                        });
                                        
                                        self.resting_players.push( RestingPlayer {
                                            player: player1,
                                            ready_at,
                                        });
                                    } else if player1.star >= self.target_star {
                                        self.winners.push(PlayerInfo{
                                            id: player1.id,
                                            name: player1.name.clone(),
                                            status: player1.star,
                                        });
                                        self.players.push(player1);
                                    } else {
                                        self.losers.push(PlayerInfo{
                                            id: player1.id,
                                            name: player1.name.clone(),
                                            status: player1.star,
                                        });
                                        self.players.push(player1);
                                    }

                                    // Player 2
                                    if !player2.cards.is_empty() && player2.star > 0 {
                                        self.player_logs.push( PlayerLog { 
                                            player_id: player2.id,
                                            message:format!("{} against {} — {} vs {}",
                                                player2_result.description(),
                                                player1_name,
                                                player2_card.emoji(),
                                                player1_card.emoji(),
                                            ),
                                        });
                                        
                                        self.resting_players.push( RestingPlayer {
                                            player: player2,
                                            ready_at,
                                        });
                                    } else if player2.star >= self.target_star {
                                        self.player_logs.push( PlayerLog { 
                                            player_id: ALL_PLAYER_ID,
                                            message:format!("{} won with {} stars!",
                                                player2.name,
                                                player2.star),
                                        });

                                        self.winners.push(PlayerInfo{
                                            id: player2.id,
                                            name: player2.name.clone(),
                                            status: player2.star,
                                        });
                                        self.players.push(player2);
                                    } else {
                                        self.player_logs.push(PlayerLog {
                                            player_id: ALL_PLAYER_ID,
                                            message: format!(
                                                "{} was eliminated!",
                                                player2.name,
                                            ),
                                        });

                                        self.losers.push(PlayerInfo{
                                            id: player2.id,
                                            name: player2.name.clone(),
                                            status: player2.star,
                                        });
                                        self.players.push(player2);
                                    }
                                }
                            }  
                        }
                    }
                }
                self.ongoing_duels = remaining_duels;

                // Create new duel if possible
                if self.waiting_players.len() >= 2 {
                    let player1 = self.waiting_players.pop_front().unwrap();
                    let player2 = self.waiting_players.pop_front().unwrap();

                    let timeout_at = Instant::now() + self.duel_duration;
                    self.ongoing_duels.push(Duel::new(player1, player2, timeout_at));

                    let game_state = GameState::Playing{
                        remaining_players: self.get_remaining_player_infos(),
                        winners: self.winners.clone(),
                        playable_card_count: self.playable_card_count,
                    };
                    let player_updates = self.get_in_game_player_updates();
                    let player_logs = std::mem::take(&mut self.player_logs);
                    return (game_state, player_updates, player_logs);
                }

                // No players available to create another duel.
                if self.ongoing_duels.is_empty() && self.resting_players.is_empty() {
                    if let Some(mut player) = self.waiting_players.pop_front() {
                        // There's no player left as an opponent,
                        let remaining_cards = player.take_remaining_cards();
                        self.playable_card_count.remove(remaining_cards);

                        self.losers.push(PlayerInfo{
                            id: player.id,
                            name: player.name.clone(),
                            status: player.star,
                        });
                        self.players.push(player);
                    }
                    let restart_at = Instant::now() + self.restart_duration;
                    self.status = GameStatus::Finished(restart_at);

                    let game_state = GameState::Finished {
                        winners: self.winners.clone(),
                        remaining_secs: restart_at
                            .saturating_duration_since(Instant::now())
                            .as_secs(),
                    };
                    let player_updates = self.get_in_game_player_updates();
                    let player_logs = std::mem::take(&mut self.player_logs);
                    return (game_state, player_updates, player_logs);
                }

                
                let game_state = GameState::Playing{
                    remaining_players: self.get_remaining_player_infos(),
                    winners: self.winners.clone(),
                    playable_card_count: self.playable_card_count,
                };
                let player_updates = self.get_in_game_player_updates();
                let player_logs = std::mem::take(&mut self.player_logs);
                (game_state, player_updates, player_logs)
            },

            GameStatus::Finished(restart_at) => {
                let game_state = GameState::Finished {
                    winners: self.winners.clone(),
                    remaining_secs: restart_at
                        .saturating_duration_since(Instant::now())
                        .as_secs(),
                };
                let player_updates = self.get_in_game_player_updates();
                let player_logs = std::mem::take(&mut self.player_logs);

                if Instant::now() >= restart_at {self.status = GameStatus::Waiting}

                (game_state, player_updates, player_logs)
            },
        }
    }

    fn get_remaining_player_infos(&self) -> Vec<PlayerInfo> {
        let mut remaining_players = Vec::new();

        let waiting_players: Vec<PlayerInfo> = self.waiting_players.iter()
            .map(|x| PlayerInfo {
                id: x.id,
                name: x.name.clone(),
                status: x.star,
            }).collect();
        remaining_players.extend(waiting_players);

        let resting_players: Vec<PlayerInfo> = self.resting_players.iter()
            .map(|x| PlayerInfo {
                id: x.player.id,
                name: x.player.name.clone(),
                status: x.player.star,
            }).collect();
        remaining_players.extend(resting_players);
       
        let in_duel_players: Vec<PlayerInfo> = self.ongoing_duels.iter()
            .flat_map(|duel| [&duel.player1, &duel.player2])
            .map(|player| PlayerInfo{
                id: player.id,
                name: player.name.clone(),
                status: player.star
            }).collect();
        remaining_players.extend(in_duel_players);

        remaining_players
    }

    fn get_in_game_player_updates(&self) -> Vec<PlayerUpdate> {
        let mut player_updates = Vec::new();

        let waiting_players: Vec<PlayerUpdate> = self.waiting_players.iter()
            .map(|x| PlayerUpdate {
                player_id: x.id,
                state: PlayerState::Waiting {
                    star: x.star,
                    playable_card_count: CardCount::from_cards(&x.cards)
                },
            }).collect();
        player_updates.extend(waiting_players);

        let resting_players: Vec<PlayerUpdate> = self.resting_players.iter()
            .map(|x| PlayerUpdate {
                player_id: x.player.id,
                state: PlayerState::Resting {
                    star: x.player.star,
                    playable_card_count: CardCount::from_cards(&x.player.cards),
                    remaining_secs: x.ready_at
                        .saturating_duration_since(Instant::now())
                        .as_secs(),
                },
            }).collect();
        player_updates.extend(resting_players);
       
        let mut in_duel_players = Vec::new();
        for duel in &self.ongoing_duels {
            let remaining_secs = duel.timeout_at
                .saturating_duration_since(Instant::now())
                .as_secs();

            
            for player in [&duel.player1, &duel.player2] {
                let opponent = if player.id == duel.player1.id {
                    &duel.player2
                } else {
                    &duel.player1
                };

                let has_submitted_card = if player.id == duel.player1.id {
                    matches!(duel.player1_card, Availability::Available(_))
                } else {
                    matches!(duel.player2_card, Availability::Available(_))
                };

                in_duel_players.push(PlayerUpdate {
                    player_id: player.id,
                    state: PlayerState::InDuel {
                        star: player.star,
                        playable_card_count: CardCount::from_cards(&player.cards),
                        opponent_name: opponent.name.clone(),
                        remaining_secs,
                        has_submitted_card,
                    },
                });
            }

        }
        player_updates.extend(in_duel_players);

        let winners: Vec<PlayerUpdate> = self.winners.iter()
            .map(|x| PlayerUpdate {
                player_id: x.id,
                state: PlayerState::Winning {star: x.status},
            }).collect();
        player_updates.extend(winners);

        let losers: Vec<PlayerUpdate> = self.losers.iter()
            .map(|x| PlayerUpdate {
                player_id: x.id,
                state: PlayerState::Losing,
            }).collect();
        player_updates.extend(losers);

        player_updates
    }
}


#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PlayerInfo {
    pub id: usize,
    pub name: String,
    pub status: usize, // can be ready status (1/0), or star status (todo: refactor)
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum GameState {
    Waiting{player_infos: Vec<PlayerInfo>},
    Playing{
        remaining_players: Vec<PlayerInfo>,
        winners: Vec<PlayerInfo>,
        playable_card_count: CardCount,
    },
    Finished{winners: Vec<PlayerInfo>, remaining_secs: u64}
}

pub struct PlayerLog {
    pub player_id: usize,
    pub message: String,
}

pub struct PlayerUpdate{
    pub player_id: usize,
    pub state: PlayerState,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum PlayerState { // consider making this as player property
    Joined, 
    Waiting { // For next duel
        star: usize,
        playable_card_count: CardCount,
    },
    InDuel {
        star: usize,
        playable_card_count: CardCount,
        opponent_name: String,
        remaining_secs: u64,
        has_submitted_card: bool,
    },
    Resting { // From duel
        star: usize,
        playable_card_count: CardCount,
        remaining_secs: u64,
    },
    Winning {star: usize},
    Losing,
}


#[derive(Debug, Copy, Clone, serde::Serialize, serde::Deserialize)]
pub struct CardCount {
    pub rock: usize,
    pub paper: usize,
    pub scissors: usize,
}

impl CardCount {
    fn new(initial_count: usize) -> CardCount {
        CardCount{
            rock: initial_count,
            paper: initial_count,
            scissors: initial_count,
        }
    }

    fn from_cards(cards: &Vec<Card>) -> CardCount {
        let mut rock = 0;
        let mut paper = 0;
        let mut scissors = 0;

        for card in cards {
            match card {
                Card::Rock => rock += 1,
                Card::Paper => paper += 1,
                Card::Scissors => scissors += 1,
            }
        }

        CardCount{rock,paper,scissors} 
    }

    fn reset(&mut self, initial_count: usize) {
        self.rock = initial_count;
        self.paper = initial_count;
        self.scissors = initial_count;
    }

    fn total(&self) -> usize {
        self.rock + self.paper + self.scissors
    }

    pub fn remove(&mut self, cards: Vec<Card>) {
        for card in cards {
            match card {
                Card::Rock => self.rock -= 1,
                Card::Paper => self.paper -= 1,
                Card::Scissors => self.scissors -= 1,
            }
        }
    }

    
    pub fn max(&self) -> usize {
        self.rock.max(self.paper).max(self.scissors)
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_player_works_when_game_is_not_started() {
        let mut not_starting_game = Game::new();

        let new_player = Player::new(
            1,
            "player1".to_string(),
            RandomStrategy,
        );

        not_starting_game.add_player(new_player);
        
        assert!(not_starting_game.players.len() == 1);
    }
}
