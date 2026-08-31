use std::io;
use std::net::{
    IpAddr,
    SocketAddr,
    UdpSocket,
};

use crate::constant::{HOST_PORT, CLIENT_PORT};
use crate::game::{GameState, PlayerState, CardCount};
use crate::network::{HostMessage, ClientMessage, encode, decode};
use crate::ui::{TermUi, KeyResult, PlayerDisplay};


pub struct Client {
   term_ui: TermUi,
   is_quit: bool,
   state: PlayerState, 
}

impl Client {
    pub fn new() -> io::Result<Self> {
        let client = Self {
            term_ui: TermUi::new()?,
            is_quit: false,
            state: PlayerState::Joined
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
                        match &self.state {
                            PlayerState::Joined => {
                                match input.to_lowercase().as_str() {
                                   "y" => {
                                        let msg = ClientMessage::SetReady(true);
                                        _ = socket.send(&encode(&msg));

                                   },
                                   "n" => {
                                        let msg = ClientMessage::SetReady(false);
                                        _ = socket.send(&encode(&msg));
                                   },
                                   _ => (),
                               };
                            },

                            PlayerState::Waiting {..} => {
                                
                            },

                            PlayerState::InDuel { 
                                star,
                                playable_card_count,
                                remaining_secs
                            } => {

                            },

                            PlayerState::Resting {
                                star,
                                playable_card_count,
                                last_duel_description,
                                remaining_secs,
                            } => {
                                if let "y" = input.to_lowercase().as_str() {
                                    let msg = ClientMessage::ReadyForNextDuel;
                                    _ = socket.send(&encode(&msg));
                                }
                            },

                            PlayerState::Winning { star } => {
                                todo!()
                            },

                            PlayerState::Losing => {
                                todo!()
                            },
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

                        self.term_ui.set_top_title("Room")?;
                        self.term_ui.set_top_subtitle("Waiting for players...")?;

                        let players_title = format!("Players ({}/{}): ",
                            ready_count, players_display.len()
                        );
                        self.term_ui.set_players_l(&players_title, &players_display)?;
                    },

                    GameState::Playing {
                        remaining_players,
                        winners,
                        playable_card_count
                    } => {
                        self.term_ui.set_top_title("Playable Cards:")?;
                        let card_count_desc = format!("Rock: {}| Paper: {}| Scissors {}",
                            playable_card_count.rock,
                            playable_card_count.paper,
                            playable_card_count.scissors,
                        );
                        self.term_ui.set_top_subtitle(&card_count_desc)?;

                        let mut winners_display = Vec::new(); 
                        for player_info in winners {
                            let mut status = "*".repeat(player_info.status);
                            status.push_str(&format!("({})", player_info.status));
                            
                            if player_info.id == 1 {status.push_str(" [You]");}

                            winners_display.push({ PlayerDisplay {
                                name: player_info.name, status
                            }})
                        }
                        self.term_ui.set_players_l("Winners: ", &winners_display)?;

                        let mut players_display = Vec::new(); 
                        for player_info in remaining_players {
                            let mut status = "*".repeat(player_info.status);
                            status.push_str(&format!("({})", player_info.status));
                            
                            if player_info.id == 1 {status.push_str(" [You]");}

                            players_display.push({ PlayerDisplay {
                                name: player_info.name, status
                            }})
                        }
                        self.term_ui.set_players_r("Remaining Players: ", &players_display)?;
                    },
                    
                    GameState::Finished {winners} => {
                        todo!();
                    }
                }

                self.state = player_state;
                match &self.state {
                    PlayerState::Joined => {
                        self.term_ui.set_instruction(
                            "Ready: [Y] Yes, [N] No"
                        )?;
                    },

                    PlayerState::Waiting { star, playable_card_count } => {
                        Self::set_player_stats(
                            &mut self.term_ui,
                            *star,
                            playable_card_count,
                        )?;

                        self.term_ui.set_instruction(
                            "Waiting for other players to duel..."
                        )?;
                    },

                    PlayerState::InDuel { 
                        star,
                        playable_card_count,
                        remaining_secs
                    } => {
                        Self::set_player_stats(
                            &mut self.term_ui,
                            *star,
                            playable_card_count,
                        )?;

                        let instruction = format!(
                            "Choose Card to Play: [R] Rock, [P] Paper,  [S] Scissors,  |  Time: {}s",
                            remaining_secs,
                        );
                        self.term_ui.set_instruction(&instruction)?;
                    },

                    PlayerState::Resting {
                        star,
                        playable_card_count,
                        last_duel_description,
                        remaining_secs,
                    } => {
                        Self::set_player_stats(
                            &mut self.term_ui,
                            *star,
                            playable_card_count,
                        )?;

                        let instruction = format!(
                            "{}  |  Ready for the next duel? [Y] Yes  |  {}s",
                            last_duel_description,
                            remaining_secs,
                        );

                        self.term_ui.set_instruction(&instruction)?;
                    },

                    PlayerState::Winning { star } => {
                        todo!()
                    },

                    PlayerState::Losing => {
                        todo!()
                    },
                }
            }
        }
        Ok(())
    }

    fn set_player_stats(
        term_ui: &mut TermUi,
        star: usize,
        playable_card_count: &CardCount,
    ) -> io::Result<()> {
        let mut star_str = "*".repeat(star);
        star_str.push_str(&format!("({})", star));

        term_ui.set_bot_title(&format!("Stars: {} | Your Cards:", star_str))?;

        let card_count_desc = format!(
            "Rock: {} | Paper: {} | Scissors: {}",
            playable_card_count.rock,
            playable_card_count.paper,
            playable_card_count.scissors,
        );

        term_ui.set_bot_subtitle(&card_count_desc)?;

        Ok(())
    }

}
