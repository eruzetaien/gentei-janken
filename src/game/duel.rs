use super::player; 

pub struct Duel {
    pub player1: player::Player,
    pub player2: player::Player,
}

impl Duel {
    pub fn play(&mut self) {
        let mut player1_card = None;

        player1_card = self.player1.set_card_to_play();
    }
}
