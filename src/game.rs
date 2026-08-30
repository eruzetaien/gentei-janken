use std::{ collections::VecDeque, mem};
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

pub struct Game {
    players: Vec<Player>,
    is_playing: bool,
    playable_card_count: CardCount,
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
            is_playing: false,
            playable_card_count: CardCount::new(0)
        }
    }
    pub fn add_players(&mut self, mut players: Vec<Player>) {
        if self.is_playing {return;}

        self.players.append(&mut players);
    }
    pub fn add_player(&mut self, player: Player) {
        if self.is_playing {return;}

        self.players.push(player);
    }

    pub fn remove_player(&mut self, player_id: usize) {
        if self.is_playing {
            return;
        }

        self.players.retain(|player| {player.id != player_id})
    }

    pub fn play(mut self) {
        let star = 3;
        let target_star = 5;
        let total_player_card_per_type = 4;
        let total_player = self.players.len();
        
        println!("Init players");
        for player in &mut self.players {
            let mut initial_cards = Vec::new();
            for _ in 0..total_player_card_per_type {
                initial_cards.push(Card::Rock);
                initial_cards.push(Card::Paper);
                initial_cards.push(Card::Scissors);
            }

            player.init(star, initial_cards);
        }
        self.is_playing = true;
        
        // Loop until all cards are played
        let total_card_per_type = total_player_card_per_type * total_player;
        self.playable_card_count.reset(total_card_per_type);

        let mut waiting_players: VecDeque<Player> =
            mem::take(&mut self.players).into();

        let mut winners = Vec::new();
        let mut losers = Vec::new();

        println!("Game start!");
        let mut ongoing_duels: Vec<Duel> = Vec::new();
        
        while self.playable_card_count.total() > 0  { 

            let mut remaining_duels = Vec::new();
            for duel in ongoing_duels.drain(..) {
                match duel.play(&self.playable_card_count) {
                    DuelStatus::Waiting(duel) => remaining_duels.push(duel),
                    DuelStatus::Finished(duel_result) => {
                        self.playable_card_count.remove(duel_result.returned_cards);

                        for player in [duel_result.player1, duel_result.player2] {
                            if !player.cards.is_empty() && player.star > 0 {
                                waiting_players.push_back(player);
                            } else if player.star >= target_star {
                                println!( "{:?} win with star: {:?}",
                                    &player.name,
                                    &player.star,
                                );
                                winners.push(player);
                            } else {
                                println!( "{:?} lose", &player.name);
                                losers.push(player);
                            }
                        }    
                    }
                }
            }
            ongoing_duels = remaining_duels;

            // Create new duel if possible
            if waiting_players.len() >= 2 {
                let player1 = waiting_players.pop_front().unwrap();
                let player2 = waiting_players.pop_front().unwrap();

                ongoing_duels.push(Duel {
                    player1,
                    player2,
                });

                continue;
            }

            // No players available to create another duel.
            if ongoing_duels.is_empty() {
                if let Some(mut player) = waiting_players.pop_front() {
                    println!("There's no player left as an opponent for {:?}",
                        &player.name);
                    let remaining_cards = player.take_remaining_cards();
                    self.playable_card_count.remove(remaining_cards);

                    losers.push(player);
                }
                break; 
            }
        }
        println!("\nGame finished!");

        let mut total_player_star = 0;
        println!();
        println!("Winners: {}", winners.len());
        for player in &winners {
            total_player_star += player.star;
            println!("  - {:?} ({} stars)", player.name, player.star);
        }

        println!("Losers: {}", losers.len());
        for player in &losers {
            total_player_star += player.star;
            println!("  - {:?} ({} stars)", player.name, player.star);
        }

        assert_eq!(total_player_star, total_player * star);
    }

    pub fn game_state(&self) -> GameState {
        if self.is_playing {
            return GameState::Playing { 
                playable_card_count: self.playable_card_count.clone()
            }
        }
        let player_infos = self.players
            .iter()
            .map(|x| PlayerInfo {
                id: x.id,
                name: x.name.clone(),
                status: 0,
            })
            .collect();
        GameState::Waiting { player_infos }
    }

    pub fn player_state(&self) -> &str {
        if self.is_playing { "the game is on"}
        else {"Waiting for players..."}
    }
}

pub enum Availability<T> {
    Available(T),
    Waiting,
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
    Playing{playable_card_count: CardCount}, 
}

pub enum PlayerState {
    Idle,
    Playing{playable_card_count: CardCount}
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
        let mut not_starting_game = Game {
            players: Vec::new(),
            is_playing: false,
            playable_card_count: CardCount::new(0),
        };

        let new_player = Player::new(
            1,
            "player1".to_string(),
            RandomStrategy,
        );

        not_starting_game.add_player(new_player);
        
        assert!(not_starting_game.players.len() == 1);
    }
}
