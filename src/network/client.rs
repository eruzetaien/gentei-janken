use std::io;
use std::net::{
    IpAddr,
    SocketAddr,
    UdpSocket,
};

use crate::constant::{HOST_PORT, CLIENT_PORT};
use crate::game::{GameState, CardCount};
use crate::network::{HostMessage, ClientMessage, encode, decode};
use crate::ui::{TermUi, KeyResult, PlayerDisplay};

pub enum ClientState {
    InRoom(bool), // ready or not
}

pub struct Client {
   term_ui: TermUi,
   is_quit: bool,
   state: ClientState, 
}

impl Client {
    pub fn new() -> io::Result<Self> {
        let client = Self {
            term_ui: TermUi::new()?,
            is_quit: false,
            state: ClientState::InRoom(false)
        };
        Ok(client)
    } 

    pub fn start(&mut self, host_ip: IpAddr, player_name: String) -> io::Result<()> {
        let socket = self.bind_socket()?;

        let host_socket = SocketAddr::new(host_ip, HOST_PORT);
        socket.connect(host_socket)?;

        let join_msg = ClientMessage::Join{player_name};
        let _size = socket.send(&encode(&join_msg))?;

        let mut buf = [0u8; 1024];
        socket.set_nonblocking(true)?;
        while !self.is_quit {
            loop {
                match socket.recv_from(&mut buf) {
                    Ok((size, _addr)) => {
                        let host_msg: HostMessage = decode(&buf[..size]);
                        self.handle_message(host_msg)?; 
                    },

                    Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                        break;
                    },

                    Err(e) => return Err(e),
                }
                
            }

            if let Some(key_result) = self.term_ui.poll_key()? {
                match key_result {
                    KeyResult::Exit => {
                        let msg = ClientMessage::Disconnect;
                           _ = socket.send(&encode(&msg));

                        self.is_quit = true;
                    }
                    KeyResult::Submitted(input) => {
                       if let ClientState::InRoom(is_ready) = self.state {
                           let is_ready = match input.to_lowercase().as_str() {
                               "y" => true,
                               "n" => false,
                               _ => is_ready,
                           };

                           let msg = ClientMessage::SetReady(is_ready);
                           _ = socket.send(&encode(&msg));
                       }
                    }
                }
            };
        }

        self.term_ui.restore()?;
        Ok(())
    }

    fn bind_socket(&self) -> io::Result<UdpSocket>{
        // let client_addr = format!("0.0.0.0:{CLIENT_PORT}");
        let client_addr = "0.0.0.0:0"; // for debug
        let socket = UdpSocket::bind(client_addr)?;

        Ok(socket)
    }


    fn handle_message(&mut self, message: HostMessage) -> io::Result<()>{
        match message {
            HostMessage::Update {game_state, player_state} => {
                match game_state {
                    GameState::Waiting {player_infos} => {
                        let mut players_display = Vec::new(); 
                        let mut ready_count = 0;
                        for player_info in player_infos {
                            let mut status = String::new(); 
                            if player_info.status == 1 {
                                status.push_str("Ready");
                                ready_count += 1;
                            } else {status.push_str("Not Ready");}
                            
                            if player_info.id == 1 {status.push_str(" [You]");}

                            players_display.push({ PlayerDisplay {
                                name: player_info.name, status
                            }})
                        }

                        let players_title = format!("Players ({}/{}): ",
                            ready_count, players_display.len()
                        );
                        self.term_ui.set_top_title("Room")?;
                        self.term_ui.set_top_subtitle("Waiting for players...")?;
                        self.term_ui.set_players_l(&players_title, &players_display)?;

                        self.term_ui.set_instruction(
                            "Ready: [Y] Yes, [N] No"
                        )?;
                    },

                    GameState::Playing {
                        remaining_players,
                        winners,
                        playable_card_count
                    } => {
                        println!("{:?}", playable_card_count);
                    },
                    
                    GameState::Finished {winners} => {
                        todo!();
                    }
                }
            }
        }
        Ok(())
    }
}
