use std::collections::VecDeque;
use std::mem;

pub mod card;
pub mod player;
pub mod duel;

pub struct Game {
    players: Vec<player::Player>,
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
    pub fn add_player(&mut self, player: player::Player) {
        if self.is_playing {return;}

        self.players.push(player);
    }

    pub fn play(mut self) {
        let star = 3;
        let target_star = 5;
        let total_card_per_type = 4;
        
        println!("Init players");
        for player in &mut self.players {
            let mut initial_cards = Vec::new();
            for _ in 0..total_card_per_type {
                initial_cards.push(card::Card::Rock);
                initial_cards.push(card::Card::Paper);
                initial_cards.push(card::Card::Scissors);
            }

            player.init(star, initial_cards);
        }
        self.is_playing = true;

        // Loop until all cards are played
        let total_playable_card_per_type = total_card_per_type * self.players.len();
        let mut playable_card_count = CardCount::new(total_playable_card_per_type);

        let mut waiting_players: VecDeque<player::Player> =
            mem::take(&mut self.players).into();

        let mut winners = Vec::new();
        let mut losers = Vec::new();

        println!("Game start!");
        while playable_card_count.total() > 0  {
            if waiting_players.len() < 2 {
                if let Some(mut player) = waiting_players.pop_front() {
                    println!("There's no player left as an opponent for {:?}",
                        &player.name);
                    let remaining_cards = player.take_remaining_cards();
                    playable_card_count.remove(remaining_cards);

                    losers.push(player);
                }
                break;
            } 
            let player1 = waiting_players.pop_front().unwrap();
            let player2 = waiting_players.pop_front().unwrap();

            let duel = duel::Duel {
                player1,
                player2,
            }; 
            let result = duel.play(&playable_card_count);
            
            playable_card_count.remove(result.returned_cards);
            
            for player in [result.player1, result.player2] {
                if !player.cards.is_empty() && player.star > 0 {
                    waiting_players.push_back(player);
                } else if player.star > target_star {
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
        println!("\nGame finished!");

        println!();
        println!("Winners: {}", winners.len());
        for player in &winners {
            println!("  - {:?} ({} stars)", player.name, player.star);
        }

        println!("Losers: {}", losers.len());
        for player in &losers {
            println!("  - {:?} ({} stars)", player.name, player.star);
        }

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

    pub fn remove(&mut self, cards: Vec<card::Card>) {
        for card in cards {
            match card {
                card::Card::Rock => self.rock -= 1,
                card::Card::Paper => self.paper -= 1,
                card::Card::Scissors => self.scissors -= 1,
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
        };

        let new_player = player::Player{
            name: "player1".to_string(),
            star: 0,
            cards: Vec::new(),
            strategy: Box::new(player::RandomStrategy{}),
        };

        not_starting_game.add_player(new_player);
        
        assert!(not_starting_game.players.len() == 1);
    }
}

