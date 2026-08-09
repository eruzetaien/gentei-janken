use gentei_janken::game;
use gentei_janken::game::player;

fn main() {
   let mut game = game::Game::new(); 

    let player1 = player::Player{
        name: "player1".to_string(),
        star: 0,
        cards: Vec::new(),
        strategy: Box::new(player::BalancedStrategy),
    };

    let player2 = player::Player{
        name: "player2".to_string(),
        star: 0,
        cards: Vec::new(),
        strategy: Box::new(player::RandomStrategy),
    };

    let player3 = player::Player{
        name: "player3".to_string(),
        star: 0,
        cards: Vec::new(),
        strategy: Box::new(player::ProbabilityStrategy),
    };
    
    let player4 = player::Player{
        name: "player4".to_string(),
        star: 0,
        cards: Vec::new(),
        strategy: Box::new(player::RandomStrategy),
    };

    game.add_player(player1); 
    game.add_player(player2); 
    game.add_player(player3); 
    game.add_player(player4); 
   
    game.play();
}
