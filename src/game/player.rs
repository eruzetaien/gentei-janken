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
        assert!(player.cards.len() == expected_len);    }
}
