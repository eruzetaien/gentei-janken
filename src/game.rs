use rand::seq::SliceRandom;

pub mod card;
pub mod player;
pub mod duel;

pub use player::Player;

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

    pub fn play(mut self) {
        let star = 3;
        let target_star = 5;
        let total_card_per_type = 4;
        let total_player = self.players.len();
        
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
        let total_playable_card_per_type = total_card_per_type * total_player;
        let mut playable_card_count = CardCount::new(total_playable_card_per_type);

        let mut waiting_players = self.players;

        let mut winners = Vec::new();
        let mut losers = Vec::new();

        println!("Game start!");
        while playable_card_count.total() > 0  {
            if waiting_players.len() < 2 {
                if let Some(mut player) = waiting_players.pop() {
                    println!("There's no player left as an opponent for {:?}",
                        &player.name);
                    let remaining_cards = player.take_remaining_cards();
                    playable_card_count.remove(remaining_cards);

                    losers.push(player);
                }
                break;
            } 
            if let Some((player1,player2)) = Self::matchmaking(&mut waiting_players){
                let duel = duel::Duel {
                    player1,
                    player2,
                }; 
                let result = duel.play(&playable_card_count);
                
                playable_card_count.remove(result.returned_cards);
                
                for player in [result.player1, result.player2] {
                    if !player.cards.is_empty() && player.star > 0 {
                        waiting_players.push(player);
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

    fn matchmaking(players: &mut Vec<Player>) -> Option<(Player,Player)> {
        if players.len() < 2 {
            return None;
        }

        players.shuffle(&mut rand::rng());

        let player1 = players.pop().unwrap();
        let player2 = players.pop().unwrap();

        Some((player1,player2))
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

