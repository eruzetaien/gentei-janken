use std::mem;

use super::card; 
use super::CardCount;

pub trait PlayStrategy {
    fn choose_card(
        &self,
        cards: &mut Vec<card::Card>,
        returned_cards: &CardCount
    ) -> card::Card;
}

pub struct RandomStrategy;
pub struct BalancedStrategy;
pub struct ProbabilityStrategy;

impl PlayStrategy for RandomStrategy {
    fn choose_card(
        &self,
        cards: &mut Vec<card::Card>,
        _returned_cards: &CardCount
    ) -> card::Card 
    {
        let rand_index = rand::random_range(0..cards.len()); 
        cards.remove(rand_index)
    }
}

pub struct Player {
    pub name: String,
    pub star: usize,
    pub cards: Vec<card::Card>,
    pub strategy: Box<dyn PlayStrategy>,
}

impl Player {
    pub fn init(&mut self, star: usize, cards: Vec<card::Card>) {
        self.star = star;
        self.cards = cards;
    }

    pub fn set_card_to_play(&mut self, returned_cards: &CardCount ) -> card::Card{
        assert!(!self.cards.is_empty());
        
        self.strategy.choose_card(&mut self.cards, returned_cards)
    }

    pub fn win_duel(&mut self) {
        self.star += 1;
    }

    pub fn lose_duel(&mut self) -> Vec<card::Card> {
        self.star -= 1;

        if self.star == 0 {
            return mem::take(&mut self.cards);
        }

        vec![] 
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
            strategy: Box::new(RandomStrategy),
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
    fn random_strategy_works() {
        let mut player = Player {
            name: "player1".to_string(),
            star: 0,
            cards: vec![
                card::Card::Rock,
                card::Card::Paper,
                card::Card::Scissors,
            ],
            strategy: Box::new(RandomStrategy),
        };

        let card_counts = CardCount{rock:0, paper:0, scissors:0}; 
        let _selected_card = player.set_card_to_play(&card_counts);

        assert_eq!(player.cards.len(), 2);
    }

}
