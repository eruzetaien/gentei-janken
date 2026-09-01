use serde::ser;
use serde::de;

use crate::game::Card;
use crate::game::{GameState,PlayerState};

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub enum ClientMessage {
    Join{player_name: String},
    SetReady(bool), 
    Disconnect,
    ReadyForNextDuel,
    ChooseCardToPlay(Card),
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub enum HostMessage {
    Update {
        game_state: GameState,
        player_state: PlayerState,
        player_logs: Vec<String>,
    }
}

pub fn encode<T: ser::Serialize>(message: &T) -> Vec<u8> {
    postcard::to_stdvec(message)
        .expect("Failed to serialize protocol message")
}

pub fn decode<'a, T: de::Deserialize<'a>>(bytes: &'a [u8]) ->  T{
    postcard::from_bytes(bytes)
        .expect("Failed to deserialize protocol message")
}
