use std::io;
use std::net::{
    IpAddr,
    SocketAddr,
    UdpSocket,
};
use std::sync::mpsc::Sender;

use crate::player_loader::load_players_from_json;
use crate::game::Game;
use crate::constant::HOST_PORT;
use crate::network::{ClientMessage, HostMessage, encode, decode};
pub struct Host {
}

impl Host {

    pub fn start(ready_tx: Sender<()>) -> io::Result<()> {
        
        // let mut game = Game::new(); 
        // let players = load_players_from_json("data/players.json");
        //
        // game.add_players(players);
        //
        // game.play();

        let host_addr = format!("0.0.0.0:{HOST_PORT}");
        let socket = UdpSocket::bind(host_addr)?;
        println!("Host listening on port {HOST_PORT}...");

        ready_tx.send(()).expect("Failed to notify host readiness");

        let mut buf = [0u8; 1024];

        loop {
            let (size, addr) = socket.recv_from(&mut buf)?;
            println!("Received {size} bytes from {addr}");
            let client_msg: ClientMessage = decode(&buf[..size]);

            match client_msg {
                ClientMessage::Join {player_name} => {
                    println!("Player {player_name} wants to join: {addr}");
                }
            }
        }
    }
}

