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

impl PlayStrategy for BalancedStrategy {
    fn choose_card(
        &self,
        cards: &mut Vec<card::Card>,
        _returned_cards: &CardCount
    ) -> card::Card 
    {
        let mut rock_count_idx = (0, 0);
        let mut paper_count_idx = (0, 0);
        let mut scissors_count_idx = (0,0);

        for (idx, card) in cards.iter().enumerate() {
            match *card {
                card::Card::Rock => {
                    rock_count_idx.0 += 1;
                    rock_count_idx.1 = idx;
                },
                card::Card::Paper => {
                    paper_count_idx.0 += 1;
                    paper_count_idx.1 = idx;
                },
                card::Card::Scissors => {
                    scissors_count_idx.0 += 1;
                    scissors_count_idx.1 = idx;
                },
            }
        }

        let choices = [
            rock_count_idx, paper_count_idx, scissors_count_idx
        ];
        
        let max_count = choices
            .iter()
            .map(|(count,_)| *count)
            .max()
            .unwrap_or(0);

        let candidates: Vec<_> = choices
            .iter()
            .filter(|(count, _)| *count == max_count)
            .collect();

        let chosen = candidates[rand::random_range(0..candidates.len())];
        let chosen_index = chosen.1; 

        cards.remove(chosen_index)
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

        let playable_card_count = CardCount::new(0); 
        let _selected_card = player.set_card_to_play(&playable_card_count);

        assert_eq!(player.cards.len(), 2);
    }

    #[test]
    fn balanced_strategy_works() {
        let mut player = Player {
            name: "player1".to_string(),
            star: 0,
            cards: vec![
                card::Card::Rock,
                card::Card::Rock,
                card::Card::Paper,
                card::Card::Scissors,
            ],
            strategy: Box::new(BalancedStrategy),
        };

        let playable_card_count = CardCount::new(0); 
        let selected_card = player.set_card_to_play(&playable_card_count);

        assert_eq!(selected_card, card::Card::Rock);
        assert_eq!(player.cards.len(), 3);
    }
}
