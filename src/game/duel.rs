use std::time::Instant;

use super::card; 
use super::Player;
use super::RandomStrategy;
use super::CardCount;
use super::Availability::{self, Available, Waiting};

pub struct Duel {
    pub player1: Player,
    pub player2: Player,
    pub player1_card: Availability<card::Card>,
    pub player2_card: Availability<card::Card>,
    pub timeout_at: Instant,
}

pub struct DuelResult {
    pub player1: Player,
    pub player2: Player,
    pub returned_cards: Vec<card::Card>,
    pub player1_card: Availability<card::Card>,
    pub player2_card: Availability<card::Card>,
    pub player1_result: card::PlayResult,
    pub player2_result: card::PlayResult,
} 

pub enum DuelStatus {
   Waiting(Duel),
   Finished(DuelResult)
}

impl Duel {
    pub fn new(player1: Player, player2: Player, timeout_at: Instant) -> Duel {
        Duel {
            player1,
            player2,
            player1_card: Waiting,
            player2_card: Waiting,
            timeout_at,
        }
    }
    
    pub fn play(mut self, playable_card_count: &CardCount) -> DuelStatus {
        if let Waiting = self.player1_card {
            self.player1_card = self.player1.try_choose_card(playable_card_count);
        }

        if let Waiting = self.player2_card {
            self.player2_card = self.player2.try_choose_card(playable_card_count);
        }

        let (player1_card, player2_card) =
            match (&self.player1_card, &self.player2_card) {
                (Available(player1_card), Available(player2_card)) => {
                    (player1_card, player2_card)
                }

                _ if Instant::now() < self.timeout_at => {
                    return DuelStatus::Waiting(self);
                }

                _ => {
                    // Timeout: automatically choose cards for players
                    if let Waiting = self.player1_card {
                        let player_strategy = self.player1.strategy;
                        self.player1.strategy = RandomStrategy;

                        self.player1_card =
                            self.player1.try_choose_card(playable_card_count);

                        self.player1.strategy = player_strategy;
                    }

                    if let Waiting = self.player2_card {
                        let player_strategy = self.player2.strategy;
                        self.player2.strategy = RandomStrategy;

                        self.player2_card =
                            self.player2.try_choose_card(playable_card_count);

                        self.player2.strategy = player_strategy;
                    }

                    // Re-check after RandomStrategy
                    match (&self.player1_card, &self.player2_card) {
                        (Available(player1_card), Available(player2_card)) => {
                            (player1_card, player2_card)
                        }

                        _ => {
                            return DuelStatus::Finished(DuelResult {
                                player1: self.player1,
                                player2: self.player2,
                                player1_card: self.player1_card,
                                player2_card: self.player2_card,
                                returned_cards: Vec::new(),
                                player1_result: card::PlayResult::Draw,
                                player2_result: card::PlayResult::Draw,
                            });
                        }
                    }
                }
            };

        let player1_result = player1_card.play_against(player2_card);
        let player2_result = player1_result.for_opponent();

        let mut returned_cards = vec![(*player1_card).clone(), (*player2_card).clone()];
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
        
        DuelStatus::Finished(
            DuelResult { 
                player1: self.player1,
                player2: self.player2,
                player1_card: self.player1_card,
                player2_card: self.player2_card,
                returned_cards,
                player1_result,
                player2_result,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::player::PlayStrategy;
    use std::time::Duration;

    #[test]
    fn play_works() {
        let winner = Player {
            id: 1,
            name: "player1".to_string(),
            star: 1,
            cards: vec![card::Card::Paper],
            strategy: PlayStrategy::RandomStrategy, 
        };

        let loser = Player {
            id: 2,
            name: "player2".to_string(),
            star: 1,
            cards: vec![
                card::Card::Rock,
                card::Card::Rock
            ],
            strategy: PlayStrategy::RandomStrategy, 
        };

        let mut duel = Duel::new(
            winner,
            loser,
            Instant::now() + Duration::from_secs(5),
        );
        
        let playable_card_count = CardCount::new(0); 
        let result = loop {
            match duel.play(&playable_card_count){
                DuelStatus::Finished(duel_result) => break duel_result,
                DuelStatus::Waiting(ongoing_duel) => duel = ongoing_duel
            }
        };

        assert_eq!(result.player1.star, 2);
        assert_eq!(result.player2.star, 0);
        assert_eq!(result.returned_cards.len(), 2 + 1);
        assert!(result.player2.cards.is_empty());
    }
}
