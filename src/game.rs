pub mod card;
pub mod player;
pub mod duel;

pub struct Game {
    players: Vec<player::Player>,
    rock_cards: Vec<card:: Card>,
    paper_cards: Vec<card:: Card>,
    scissors_cards: Vec<card:: Card>,
    is_playing: bool,
}

impl Game {
    pub fn add_player(&mut self, player: player::Player) {
        if self.is_playing {return;}

        self.players.push(player);
    }

    pub fn play(mut self) {
        let star = 3;
        let total_card = 4;
        
        for player in &mut self.players {
            let mut initial_cards = Vec::new();
            for _ in 0..total_card {
                initial_cards.push(card::Card::Rock);
                initial_cards.push(card::Card::Paper);
                initial_cards.push(card::Card::Scissors);
            }

            player.init(star, initial_cards);
        }

        self.is_playing = true;

        // Loop until all cards are played
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_player_works_when_game_is_not_started() {
        let mut not_starting_game = Game {
            players: Vec::new(),
            rock_cards: Vec::new(),
            paper_cards: Vec::new(),
            scissors_cards: Vec::new(),
            is_playing: false,
        };

        let new_player = player::Player{
            name: "player1".to_string(),
            star: 0,
            cards: Vec::new(),
        };

        not_starting_game.add_player(new_player);
        
        assert!(not_starting_game.players.len() == 1);
    }
}

