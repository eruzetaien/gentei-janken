use std::io;
use std::net::{
    IpAddr,
    SocketAddr,
    UdpSocket,
};

use crate::constant::{HOST_PORT, CLIENT_PORT};
use crate::game::{GameState, CardCount};
use crate::network::{HostMessage, ClientMessage, encode, decode};
pub struct Client {
}

impl Client {
    pub fn new() -> Self {
        Self {
        }
    } 

    pub fn start(&mut self, host_ip: IpAddr, player_name: String) -> io::Result<()> {
        let socket = self.bind_socket()?;

        let host_socket = SocketAddr::new(host_ip, HOST_PORT);
        socket.connect(host_socket)?;
        println!("Client connected");

        let join_msg = ClientMessage::Join{player_name};
        let size = socket.send(&encode(&join_msg))?;
        println!("Sent {size} bytes");

        let mut buf = [0u8; 1024];
        loop {
            loop {
                match socket.recv_from(&mut buf) {
                    Ok((size, _addr)) => {
                        let host_msg: HostMessage = decode(&buf[..size]);
                        self.handle_message(host_msg); 
                    },

                    Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                        break;
                    },

                    Err(e) => return Err(e),
                }
                
            }

            let (size, addr) = socket.recv_from(&mut buf)?;
            println!("Received {size} bytes from {addr}");
        }
    }

    fn bind_socket(&self) -> io::Result<UdpSocket>{
        // let client_addr = format!("0.0.0.0:{CLIENT_PORT}");
        let client_addr = "0.0.0.0:0"; // for debug
        let socket = UdpSocket::bind(client_addr)?;

        Ok(socket)
    }


    fn handle_message(&mut self, message: HostMessage) {
        match message {
            HostMessage::Update {game_state} => {
                match game_state {
                    GameState::Waiting {player_names} => {
                        println!("{:?}", player_names);
                    },
                    GameState::Playing {playable_card_count} => {
                        println!("{:?}", playable_card_count);
                    }
                }
            }
        }
    }
}
