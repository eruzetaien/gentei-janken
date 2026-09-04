use rand::seq::SliceRandom;
use std::{mem, thread, time};

use super::card::Card; 
use super::CardCount;

pub trait PlayStrategy {
    fn choose_card(
        &self,
        cards: &mut Vec<Card>,
        playable_card_count: &CardCount
    ) -> Card;
}

pub struct RandomStrategy;
pub struct BalancedStrategy;
pub struct ProbabilityStrategy;
pub struct MetaRandomStrategy {
    strategies: [Box<dyn PlayStrategy + Send>; 3],
}

pub struct Player {
    pub name: String,
    pub strategy: Box<dyn PlayStrategy + Send>,
    pub star: usize,
    pub cards: Vec<Card>,
}

impl Player {
    pub fn new(name: String, strategy: Box<dyn PlayStrategy + Send>) -> Player {
        Player {
            name,
            strategy, 
            star: 0,
            cards: Vec::new(),
        }
    }
    pub fn init(&mut self, star: usize, cards: Vec<Card>) {
        self.star = star;
        self.cards = cards;
    }

    pub fn set_card_to_play(&mut self, returned_cards: &CardCount ) -> Card{
        assert!(!self.cards.is_empty());
       
        let think_time = rand::random_range(0..3);
        thread::sleep(time::Duration::from_secs(think_time));

        self.strategy.choose_card(&mut self.cards, returned_cards)
    }

    pub fn win_duel(&mut self) {
        self.star += 1;
    }

    pub fn lose_duel(&mut self) -> Vec<Card> {
        self.star -= 1;

        if self.star == 0 {
            return mem::take(&mut self.cards);
        }

        vec![] 
    }

    pub fn take_remaining_cards(&mut self) -> Vec<Card> {
        mem::take(&mut self.cards)
    }
}

impl PlayStrategy for RandomStrategy {
    fn choose_card(
        &self,
        cards: &mut Vec<Card>,
        _playable_card_count: &CardCount
    ) -> Card 
    {
        let rand_index = rand::random_range(0..cards.len()); 
        cards.remove(rand_index)
    }
}

impl PlayStrategy for BalancedStrategy {
    fn choose_card(
        &self,
        cards: &mut Vec<Card>,
        _playable_card_count: &CardCount
    ) -> Card 
    {
        let mut choices = [
            (0,None), // rock (count, index)
            (0,None), // paper (count, index)
            (0,None), // scissors (count, index)
        ];

        for (idx, card) in cards.iter().enumerate() {
            match *card {
                Card::Rock => {
                    choices[0].0 += 1;
                    choices[0].1 = Some(idx);
                },
                Card::Paper => {
                    choices[1].0 += 1;
                    choices[1].1 = Some(idx);
                },
                Card::Scissors => {
                    choices[2].0 += 1;
                    choices[2].1 = Some(idx);
                },
            }
        }

        let choices: Vec<(usize, usize)> = choices
            .into_iter()
            .filter_map(|(count,opt_idx)| {
                opt_idx.map(|index| (count, index)) // map filter None
            })
            .collect();

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

impl PlayStrategy for ProbabilityStrategy {
    fn choose_card(
        &self,
        cards: &mut Vec<Card>,
        playable_card_count: &CardCount
    ) -> Card {
        let max_count = playable_card_count.max(); 
        let mut highest_priority_cards = Vec::new();
        let mut medium_priority_cards = Vec::new();
        let mut lowest_priority_cards = Vec::new();

        if playable_card_count.rock == max_count {
            highest_priority_cards.push(Card::Paper);
            medium_priority_cards.push(Card::Rock);
            lowest_priority_cards.push(Card::Scissors);
        }

        if playable_card_count.paper == max_count {
            highest_priority_cards.push(Card::Scissors);
            medium_priority_cards.push(Card::Paper);
            lowest_priority_cards.push(Card::Rock);
        }

        if playable_card_count.scissors == max_count {
            highest_priority_cards.push(Card::Rock);
            medium_priority_cards.push(Card::Scissors);
            lowest_priority_cards.push(Card::Paper);
        }

        let mut rng = rand::rng();
        highest_priority_cards.shuffle(&mut rng);
        medium_priority_cards.shuffle(&mut rng);
        lowest_priority_cards.shuffle(&mut rng);

        let mut priority_cards = highest_priority_cards;
        priority_cards.extend(medium_priority_cards);
        priority_cards.extend(lowest_priority_cards);
        
        for card in priority_cards {
            if let Some(index) = cards.iter().position(|x| *x == card) {
                let chosen_card = cards.remove(index);
                return chosen_card;
            }
        }

        cards.pop().expect("cards should never be empty")
    }
}

impl Default for MetaRandomStrategy {
    fn default() -> Self {
        Self::new()
    }
}

impl MetaRandomStrategy {
    pub fn new() -> MetaRandomStrategy {
        MetaRandomStrategy {
            strategies: [
                Box::new(RandomStrategy) as Box<dyn PlayStrategy + Send>,
                Box::new(BalancedStrategy) as Box<dyn PlayStrategy + Send>,
                Box::new(ProbabilityStrategy) as Box<dyn PlayStrategy + Send>,
            ]
        }
    }
}

impl PlayStrategy for MetaRandomStrategy {
    fn choose_card(
        &self,
        cards: &mut Vec<Card>,
        playable_card_count: &CardCount
    ) -> Card {
        let rand_index = rand::random_range(0..self.strategies.len()); 
        self.strategies[rand_index].choose_card(cards, playable_card_count)
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
            initial_cards.push(Card::Rock);
            initial_cards.push(Card::Paper);
            initial_cards.push(Card::Scissors);
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
                Card::Rock,
                Card::Paper,
                Card::Scissors,
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
                Card::Rock,
                Card::Rock,
                Card::Paper,
                Card::Scissors,
            ],
            strategy: Box::new(BalancedStrategy),
        };

        let playable_card_count = CardCount::new(0); 
        let selected_card = player.set_card_to_play(&playable_card_count);

        assert_eq!(selected_card, Card::Rock);
        assert_eq!(player.cards.len(), 3);
    }

    #[test]
    fn probability_strategy_works() {
        let mut player = Player {
            name: "player1".to_string(),
            star: 0,
            cards: vec![
                Card::Rock,
                Card::Paper,
                Card::Scissors,
            ],
            strategy: Box::new(ProbabilityStrategy),
        };

        let mut playable_card_count = CardCount::new(0);  
        playable_card_count.rock = 1; 
        playable_card_count.paper = 2; 
        playable_card_count.scissors = 2; 

        let selected_card = player.set_card_to_play(&playable_card_count);

        assert_eq!(selected_card, Card::Rock);
        assert_eq!(player.cards.len(), 2);
    }
}
