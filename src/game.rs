use std::{ collections::VecDeque, mem};
use std::sync::{ Arc, RwLock};
use std::sync::mpsc::{channel, Sender, Receiver, TryRecvError};

pub mod card;
pub mod player;
pub mod duel;

use card::{Card};
pub use player::{
    Player,
    RandomStrategy,
    BalancedStrategy,
    ProbabilityStrategy,
    MetaRandomStrategy,

};
use duel::{Duel, DuelResult};
use crate::thread_pool::ThreadPool;

pub struct Game {
    players: Vec<Player>,
    is_playing: bool,
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

    pub fn play(mut self, pool: &ThreadPool) {
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
        let playable_card_count = 
            Arc::new(RwLock::new(CardCount::new(total_card_per_type)));

        let mut waiting_players: VecDeque<Player> =
            mem::take(&mut self.players).into();

        let mut winners = Vec::new();
        let mut losers = Vec::new();

        println!("Game start!");
        let mut total_duel = 0;

        let (tx, rx): (Sender<DuelResult>, Receiver<DuelResult>) = channel();
        
        while playable_card_count.read().unwrap().total() > 0  { 

            match rx.try_recv() {
                Ok(duel_result) => {
                    total_duel -= 1;
                    playable_card_count.write().unwrap()
                        .remove(duel_result.returned_cards);

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
                },
                Err(TryRecvError::Empty) => {

                },
                Err(TryRecvError::Disconnected) => {

                }
            }

            if waiting_players.len() < 2 {
                if total_duel > 0 {
                    continue;
                }

                if let Some(mut player) = waiting_players.pop_front() {
                    println!("There's no player left as an opponent for {:?}",
                        &player.name);
                    let remaining_cards = player.take_remaining_cards();
                    playable_card_count.write().unwrap().remove(remaining_cards);

                    losers.push(player);
                }
                break;
            }

            let player1 = waiting_players.pop_front().unwrap();
            let player2 = waiting_players.pop_front().unwrap();
            total_duel += 1;

            let tx_clone = tx.clone();
            let card_count = Arc::clone(&playable_card_count);
            pool.execute(move || {
                let duel = Duel {
                    player1,
                    player2,
                }; 

                let duel_result = duel.play(&card_count.read().unwrap());
                tx_clone.send(duel_result)
                    .expect("failed to send duel result");
            });

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

}

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
        let mut not_starting_game = Game {
            players: Vec::new(),
            is_playing: false,
        };

        let new_player = Player{
            name: "player1".to_string(),
            star: 0,
            cards: Vec::new(),
            strategy: Box::new(player::RandomStrategy{}),
        };

        not_starting_game.add_player(new_player);
        
        assert!(not_starting_game.players.len() == 1);
    }
}

