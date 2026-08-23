use std::fs;
use std::collections::HashMap;

use crate::game::{
    Player,
    PlayStrategy,
    RandomStrategy,
    BalancedStrategy,
    ProbabilityStrategy,
};

type PlayerCategory = HashMap<String, Vec<String>>;

pub fn load_players_from_json(path: &str) -> Vec<Player> {
    let file_content = fs::read_to_string(path).expect("Failed to read file");

    let player_category: PlayerCategory = serde_json::from_str(&file_content).
        expect("Failed to parse JSON");

    let mut players = Vec::new();
    
    let mut count = 999;
    for (strategy_name, player_names) in player_category {
        match strategy_name.as_str() {
            "random" => {
                for name in player_names {
                    let strategy = RandomStrategy;
                    players.push(Player::new(count-1,name,strategy));
                }
            },
            "balanced" => {
                for name in player_names {
                    let strategy = BalancedStrategy;
                    players.push(Player::new(count-1,name,strategy));
                }
            },
            "probability" => {
                for name in player_names {
                    let strategy = ProbabilityStrategy;
                    players.push(Player::new(count-1,name,strategy));
                }
            },
            "metarandom" => {
                for name in player_names {
                    let strategy = PlayStrategy::meta_random();
                    players.push(Player::new(count-1,name,strategy));
                }
            },
            _ => {
                eprintln!("Unknown strategy '{}'", strategy_name);
                continue;
            }
        };

        count -= 1;
    }
    players
}
