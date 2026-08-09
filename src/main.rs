use std::collections::HashMap;
use std::fs;
use gentei_janken::game;
use gentei_janken::game::player::{
    Player,
    RandomStrategy,
    BalancedStrategy,
    ProbabilityStrategy
};

type PlayerCategory = HashMap<String, Vec<String>>;

fn load_players_from_json(path: &str) -> Vec<Player> {
    let file_content = fs::read_to_string(path).expect("Failed to read file");

    let player_category: PlayerCategory = serde_json::from_str(&file_content).
        expect("Failed to parse JSON");

    let mut players = Vec::new();
    
    for (strategy_name, player_names) in player_category {
        match strategy_name.as_str() {
            "random" => {
                let strategy = Box::new(RandomStrategy);

                for name in player_names {
                    players.push(Player::new(name,strategy.clone()));
                }
            },
            "balanced" => {
                let strategy = Box::new(BalancedStrategy);

                for name in player_names {
                    players.push(Player::new(name,strategy.clone()));
                }
            },
            "probability" => {
                let strategy = Box::new(ProbabilityStrategy);

                for name in player_names {
                    players.push(Player::new(name,strategy.clone()));
                }
            },
            _ => {
                eprintln!("Unknown strategy '{}'", strategy_name);
                continue;
            }
        };
    }
    players
}

fn main() {
    let mut game = game::Game::new(); 
    let players = load_players_from_json("data/players.json");

    game.add_players(players);
    game.play();
}
