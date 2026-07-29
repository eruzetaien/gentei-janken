use super::card;  

pub struct Player {
    pub name: String,
    pub star: usize,
    pub cards: Vec<card::Card>,
}
