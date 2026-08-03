use std::mem;

pub mod card;
pub mod player;
pub mod duel;

pub struct Game {
    players: Vec<player::Player>,
    is_playing: bool,
}

impl Game {
    pub fn add_player(&mut self, player: player::Player) {
        if self.is_playing {return;}

        self.players.push(player);
    }

    pub fn play(mut self) {
        let star = 3;
        let total_card_per_type = 4;
        
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
        let total_played_rock_card = 0;
        let total_played_paper_card = 0;
        let total_played_scissors_card = 0;

        let mut waiting_players = mem::take(&mut self.players);

        let total_cards_per_type = total_card_per_type * self.players.len(); 
        while total_played_rock_card != total_cards_per_type
            || total_played_paper_card != total_cards_per_type
            || total_played_scissors_card != total_cards_per_type {
            let player1 = waiting_players.pop().unwrap();
            let player2 = waiting_players.pop().unwrap();

            let mut duel = duel::Duel {
                player1: player1,
                player2: player2,
            }; 
            duel.play();
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
        };

        not_starting_game.add_player(new_player);
        
        assert!(not_starting_game.players.len() == 1);
    }
}

