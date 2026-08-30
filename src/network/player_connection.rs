use std::net::SocketAddr;
use std::sync::mpsc::Sender;
use crate::game::Card; 
use crate::network::ClientState;

pub struct PlayerConnection {
    pub id: usize,
    pub addr: SocketAddr,
    pub card_tx: Sender<Card>,
    pub state: ClientState,
}
