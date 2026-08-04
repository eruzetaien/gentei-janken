use super::card; 
use super::player; 

pub struct Duel {
    pub player1: player::Player,
    pub player2: player::Player,
}

pub struct DuelResult{
    pub player1: player::Player,
    pub player2: player::Player,
    pub returned_cards: Vec<card::Card>,
} 

impl Duel {
    pub fn play(mut self) -> DuelResult {
        let player1_card = self.player1.set_card_to_play().unwrap();
        let player2_card = self.player2.set_card_to_play().unwrap();

        let player1_result = player1_card.play_against(&player2_card);
        let mut returned_cards = vec![player1_card, player2_card];
        match player1_result {
            card::PlayResult::Win => {
                self.player1.win_duel();

                let mut disposed_cards = self.player2.lose_duel();
                returned_cards.append(&mut disposed_cards);
            },
            card::PlayResult::Lose => {
                let mut disposed_cards = self.player1.lose_duel();
                returned_cards.append(&mut disposed_cards);

                self.player2.win_duel();
            },
            card::PlayResult::Draw => {}
        }

        DuelResult {
            player1: self.player1, 
            player2: self.player2, 
            returned_cards: returned_cards
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn play_works() {
        let winner = player::Player {
           name: "player1".to_string(),
           star: 1,
           cards: vec![card::Card::Paper],
        };

        let loser = player::Player {
           name: "player2".to_string(),
           star: 1,
           cards: vec![
               card::Card::Rock,
               card::Card::Rock
           ],
        };

        let duel = Duel {
            player1: winner,
            player2: loser,
        };
        
        let result = duel.play();

        assert_eq!(result.player1.star, 2);
        assert_eq!(result.player2.star, 0);
        assert_eq!(result.returned_cards.len(), 2 + 1);
    }
}
