pub use super::card;  

pub struct Player {
    name: String,
    star: usize,
    cards: Vec<card::Card>,
}
