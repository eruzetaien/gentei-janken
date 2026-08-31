use std::{ collections::VecDeque, mem};
use std::time::{Instant, Duration};
//use std::sync::mpsc::{channel, Sender, Receiver, TryRecvError};

pub mod card;
pub mod player;
pub mod duel;

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
    last_duel_description: String,
}

enum GameStatus {
    Waiting,
    Playing,
    Finished
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

    target_star: usize,
    duel_duration: Duration,
    rest_duration: Duration,
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

            target_star: 5,
            duel_duration: Duration::from_secs(5),
            rest_duration: Duration::from_secs(10),
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

    pub fn add_bots(&mut self, count: usize) {
        let bot_strategies: [fn() -> PlayStrategy; 4] = [
            || RandomStrategy,
            || BalancedStrategy,
            || ProbabilityStrategy,
            PlayStrategy::meta_random,
        ];

        let mut bot_id = 999;
        for _ in 0..count {
            let rand_index = rand::random_range(0..bot_strategies.len()); 
            let strategy = bot_strategies[rand_index]();
            let id = bot_id;
            let player_name = format!("Bot-{}", id);
            let bot = Player::new(id, player_name, strategy);

            self.add_player(bot);
            bot_id -= 1;
        }
    }

    pub fn finish_resting(&mut self, player_id: usize) {
        if let Some(index) = self.resting_players.iter()
            .position(|x| x.player.id == player_id) {
            let resting_player = self.resting_players.remove(index);
            self.waiting_players.push_back(resting_player.player);
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

        self.waiting_players = mem::take(&mut self.players).into();
        self.status = GameStatus::Playing;
    }

    pub fn update(&mut self) -> (GameState, Vec<PlayerUpdate>) {
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
                                id: x.id,
                                state: PlayerState::Joined,
                            }
                        )
                    })
                    .unzip();

                let game_state = GameState::Waiting { player_infos };
                (game_state, player_updates)
            },

            GameStatus::Playing => {
                let target_star = 5;
               
                if self.playable_card_count.total() == 0 {
                    self.status = GameStatus::Finished;

                    let game_state = GameState::Finished {winners: self.winners.clone()};
                    let player_updates = self.get_in_game_player_updates();
                    return (game_state, player_updates);
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
                                self.resting_players.push( RestingPlayer {
                                    player: duel_result.player1,
                                    ready_at,
                                    last_duel_description: description.to_string(),
                                });

                                self.resting_players.push( RestingPlayer {
                                    player: duel_result.player2,
                                    ready_at,
                                    last_duel_description: description.to_string(),
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
                                        self.resting_players.push( RestingPlayer {
                                            player: player1,
                                            ready_at,
                                            last_duel_description: format!(
                                                "{} against {} — {:?} vs {:?}",
                                                player1_result.description(),
                                                player2_name,
                                                player1_card,
                                                player2_card,
                                            ),
                                        });
                                    } else if player1.star >= target_star {
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
                                        self.resting_players.push( RestingPlayer {
                                            player: player2,
                                            ready_at,
                                            last_duel_description: format!(
                                                "{} against {} — {:?} vs {:?}",
                                                player2_result.description(),
                                                player1_name,
                                                player2_card,
                                                player1_card,
                                            ),
                                        });
                                    } else if player2.star >= target_star {
                                        self.winners.push(PlayerInfo{
                                            id: player2.id,
                                            name: player2.name.clone(),
                                            status: player2.star,
                                        });
                                        self.players.push(player2);
                                    } else {
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
                    return (game_state, player_updates);
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
                    self.status = GameStatus::Finished;
                    let game_state = GameState::Finished {winners: self.winners.clone()};
                    let player_updates = self.get_in_game_player_updates();
                    return (game_state, player_updates);

                }

                
                let game_state = GameState::Playing{
                    remaining_players: self.get_remaining_player_infos(),
                    winners: self.winners.clone(),
                    playable_card_count: self.playable_card_count,
                };
                let player_updates = self.get_in_game_player_updates();
                (game_state, player_updates)
            },

            GameStatus::Finished => {
                let game_state = GameState::Finished {winners: self.winners.clone()};
                let player_updates = self.get_in_game_player_updates();
                return (game_state, player_updates)
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
                id: x.id,
                state: PlayerState::Waiting {
                    star: x.star,
                    playable_card_count: CardCount::from_cards(&x.cards)
                },
            }).collect();
        player_updates.extend(waiting_players);

        let resting_players: Vec<PlayerUpdate> = self.resting_players.iter()
            .map(|x| PlayerUpdate {
                id: x.player.id,
                state: PlayerState::Resting {
                    star: x.player.star,
                    playable_card_count: CardCount::from_cards(&x.player.cards),
                    remaining_secs: x.ready_at
                        .saturating_duration_since(Instant::now())
                        .as_secs(),
                    last_duel_description: x.last_duel_description.clone(),
                },
            }).collect();
        player_updates.extend(resting_players);
       
        let mut in_duel_players = Vec::new();
        for duel in &self.ongoing_duels {
            let remaining_secs = duel.timeout_at
                .saturating_duration_since(Instant::now())
                .as_secs();

            for player in [&duel.player1, &duel.player2] {
                in_duel_players.push(PlayerUpdate {
                    id: player.id,
                    state: PlayerState::InDuel {
                        star: player.star,
                        playable_card_count: CardCount::from_cards(&player.cards),
                        remaining_secs,
                    },
                });
            }
        }
        player_updates.extend(in_duel_players);

        let winners: Vec<PlayerUpdate> = self.winners.iter()
            .map(|x| PlayerUpdate {
                id: x.id,
                state: PlayerState::Winning {star: x.status},
            }).collect();
        player_updates.extend(winners);

        let losers: Vec<PlayerUpdate> = self.losers.iter()
            .map(|x| PlayerUpdate {
                id: x.id,
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
    Finished{winners: Vec<PlayerInfo>}
}

pub struct PlayerUpdate{
    pub id: usize,
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
        remaining_secs: u64,
    },
    Resting { // From duel
        star: usize,
        playable_card_count: CardCount,
        last_duel_description: String,
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
