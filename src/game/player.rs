use super::card;  

pub struct Player {
    pub name: String,
    pub star: usize,
    pub cards: Vec<card::Card>,
}

impl Player {
    pub fn init(&mut self, star: usize, cards: Vec<card::Card>) {
        self.star = star;
        self.cards = cards;
    }

    pub fn set_card_to_play(&mut self) -> Option<card::Card>{
        if self.cards.is_empty() {
            return None;
        }
        
        let rand_index = rand::random_range(0..self.cards.len()); 
        let card = self.cards.remove(rand_index);

        Some(card)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn init_works() {
        let mut player = Player{
            name: "player1".to_string(),
            star: 0,
            cards: Vec::new(),
        };

        let star = 3;
        let total_card = 4;
        
        let mut initial_cards = Vec::new();
        for _ in 0..total_card {
            initial_cards.push(card::Card::Rock);
            initial_cards.push(card::Card::Paper);
            initial_cards.push(card::Card::Scissors);
        }

        let expected_len = initial_cards.len();

        // Pass the vector into init
        player.init(star, initial_cards);
        
        assert!(player.star == star);
        assert!(player.cards.len() == expected_len);
    }

    #[test]
    fn set_card_to_play_works() {
        let mut player = Player {
            name: "player1".to_string(),
            star: 0,
            cards: vec![
                card::Card::Rock,
                card::Card::Paper,
                card::Card::Scissors,
            ],
        };

        let selected_card = player.set_card_to_play();

        assert!(selected_card.is_some());
        assert_eq!(player.cards.len(), 2);
    }

}
