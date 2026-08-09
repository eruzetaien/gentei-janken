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

#[derive(Debug, Clone, Copy)]
pub struct RandomStrategy;

#[derive(Debug, Clone, Copy)]
pub struct BalancedStrategy;

#[derive(Debug, Clone, Copy)]
pub struct ProbabilityStrategy;


pub struct Player {
    pub name: String,
    pub strategy: Box<dyn PlayStrategy>,
    pub star: usize,
    pub cards: Vec<card::Card>,
}

impl Player {
    pub fn new(name: String, strategy: Box<dyn PlayStrategy>) -> Player {
        Player {
            name,
            strategy, 
            star: 0,
            cards: Vec::new(),
        }
    }
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

    pub fn take_remaining_cards(&mut self) -> Vec<card::Card> {
        mem::take(&mut self.cards)
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
        let mut choices = [
            (0,None), // rock (count, index)
            (0,None), // paper (count, index)
            (0,None), // scissors (count, index)
        ];

        for (idx, card) in cards.iter().enumerate() {
            match *card {
                card::Card::Rock => {
                    choices[0].0 += 1;
                    choices[0].1 = Some(idx);
                },
                card::Card::Paper => {
                    choices[1].0 += 1;
                    choices[1].1 = Some(idx);
                },
                card::Card::Scissors => {
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
        cards: &mut Vec<card::Card>,
        playable_card_count: &CardCount
    ) -> card::Card {
        let mut choices: Vec<(usize, usize)> = Vec::new(); 

        if let Some(idx) = cards.iter().position(|x| *x == card::Card::Rock) {
            choices.push((playable_card_count.rock, idx));
        }

        if let Some(idx) = cards.iter().position(|x| *x == card::Card::Paper) {
            choices.push((playable_card_count.paper, idx));
        }

        if let Some(idx) = cards.iter().position(|x| *x == card::Card::Scissors) {
            choices.push((playable_card_count.scissors, idx));
        }
        
        let min_count = choices
            .iter()
            .map(|(count,_)| *count)
            .min()
            .unwrap_or(0);

        let candidates: Vec<_> = choices
            .iter()
            .filter(|(count, _)| *count == min_count)
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

    #[test]
    fn probability_strategy_works() {
        let mut player = Player {
            name: "player1".to_string(),
            star: 0,
            cards: vec![
                card::Card::Rock,
                card::Card::Paper,
                card::Card::Scissors,
            ],
            strategy: Box::new(ProbabilityStrategy),
        };

        let mut playable_card_count = CardCount::new(0);  
        playable_card_count.rock = 1; 
        playable_card_count.paper = 2; 
        playable_card_count.scissors = 2; 

        let selected_card = player.set_card_to_play(&playable_card_count);

        assert_eq!(selected_card, card::Card::Rock);
        assert_eq!(player.cards.len(), 2);
    }
}
