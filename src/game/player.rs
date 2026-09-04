use rand::seq::SliceRandom;
use std::sync::mpsc::Receiver;
use std::{mem};

use super::card::Card; 
use super::CardCount;
use super::{Availability, Availability::Waiting, Availability::Available};

pub enum PlayStrategy {
    RandomStrategy,
    BalancedStrategy,
    ProbabilityStrategy,
    MetaRandomStrategy {
        strategies: [Box<PlayStrategy>; 3],
    },
    OnlineStrategy {
        card_rx: Receiver<Card>,
    }
}

pub struct Player {
    pub id: usize,
    pub name: String,
    pub strategy: PlayStrategy,
    pub star: usize,
    pub cards: Vec<Card>,
}

impl Player {
    pub fn new(
        id: usize,
        name: String, 
        strategy: PlayStrategy,
    ) -> Player {
        Player {
            id,
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

    pub fn try_choose_card(&mut self, returned_cards: &CardCount ) -> Availability<Card>{
        assert!(!self.cards.is_empty());
       
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

impl PlayStrategy {
    pub fn meta_random() -> Self {
        Self::MetaRandomStrategy {
            strategies: [
                Box::new(PlayStrategy::RandomStrategy),
                Box::new(PlayStrategy::BalancedStrategy),
                Box::new(PlayStrategy::ProbabilityStrategy),
            ],
        }
    }

    pub fn choose_card(
        &mut self,
        cards: &mut Vec<Card>,
        playable_card_count: &CardCount
    ) -> Availability<Card> {
        match self {
            PlayStrategy::RandomStrategy => {
                let rand_index = rand::random_range(0..cards.len()); 
                Available(cards.remove(rand_index))
            }

            PlayStrategy::BalancedStrategy => {
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

                Available(cards.remove(chosen_index))
            }

            PlayStrategy::ProbabilityStrategy => {
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
                        return Available(chosen_card);
                    }
                }

                Available(cards.pop().expect("cards should never be empty"))
            }

            PlayStrategy::MetaRandomStrategy { strategies } => {
                let rand_index = rand::random_range(0..strategies.len()); 
                strategies[rand_index].choose_card(cards, playable_card_count)
            }

            PlayStrategy::OnlineStrategy { card_rx: input_card } => {
                let mut chosen_card = None;

                while let Ok(card) = input_card.try_recv() {
                    chosen_card = Some(card);
                }

                match chosen_card {
                    Some(card) => {
                        cards.iter()
                            .position(|c| *c == card)
                            .map_or(Waiting, |idx| Available(cards.remove(idx)))
                    }
                    None => Waiting,
                }
            }
        }
    }
} 


#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn init_works() {
        let mut player = Player{
            id: 1,
            name: "player1".to_string(),
            star: 0,
            cards: Vec::new(),
            strategy: PlayStrategy::RandomStrategy,
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
            id: 1,
            name: "player1".to_string(),
            star: 0,
            cards: vec![
                Card::Rock,
                Card::Paper,
                Card::Scissors,
            ],
            strategy: PlayStrategy::RandomStrategy,
        };

        let playable_card_count = CardCount::new(0); 
        let _selected_card = loop {
            if let Available(card) = player.try_choose_card(&playable_card_count) {
                break card;
            }
        };
        assert_eq!(player.cards.len(), 2);
    }

    #[test]
    fn balanced_strategy_works() {
        let mut player = Player {
            id: 1,
            name: "player1".to_string(),
            star: 0,
            cards: vec![
                Card::Rock,
                Card::Rock,
                Card::Paper,
                Card::Scissors,
            ],
            strategy: PlayStrategy::BalancedStrategy,
        };

        let playable_card_count = CardCount::new(0); 
        let selected_card = loop {
            if let Available(card) = player.try_choose_card(&playable_card_count) {
                break card;
            }
        };

        assert_eq!(selected_card, Card::Rock);
        assert_eq!(player.cards.len(), 3);
    }

    #[test]
    fn probability_strategy_works() {
        let mut player = Player {
            id: 1,
            name: "player1".to_string(),
            star: 0,
            cards: vec![
                Card::Paper,
                Card::Scissors,
            ],
            strategy: PlayStrategy::ProbabilityStrategy,
        };

        let mut playable_card_count = CardCount::new(0);  
        playable_card_count.rock = 7; 
        playable_card_count.paper = 2; 
        playable_card_count.scissors = 10; 

        let selected_card = loop {
            if let Available(card) = player.try_choose_card(&playable_card_count) {
                break card;
            }
        };

        assert_eq!(selected_card, Card::Scissors);
        assert_eq!(player.cards.len(), 1);
    }
}
