
use std::collections::HashMap;
use std::io;
use std::net::{
    SocketAddr,
    UdpSocket,
};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::mpsc::{Sender, channel};
use std::time::{Duration, Instant};

use crate::constant::{
    HOST_PORT, ALL_PLAYER_ID, PLAYER_SELF, PLAYER_OTHER
};
use crate::game::{
    Game, GameState, OnlineStrategy, Player, PlayerInfo
};
use crate::network::{
    PlayerConnection,
    HostMessage, ClientMessage,
    encode, decode
};

pub struct Host {
    game: Game,
    connection_by_id: HashMap<usize, Rc<RefCell<PlayerConnection>>>,
    connection_by_addr: HashMap<SocketAddr, Rc<RefCell<PlayerConnection>>>,
    next_player_id: usize,
    logs: Vec<String>,
    player_logs: HashMap<usize, Vec<String>>,
}

impl Default for Host {
    fn default () -> Self {
        Self::new()
    }
}

impl Host {
    pub fn new() -> Self {
        Self {
            game: Game::new(),
            connection_by_id: HashMap::new(),
            connection_by_addr: HashMap::new(),
            next_player_id: 0,
            logs: Vec::new(),
            player_logs: HashMap::new()
        }
    } 

    pub fn start(&mut self, ready_tx: Sender<()>) -> io::Result<()> {
        
        let socket = self.bind_socket()?;

        ready_tx.send(()).expect("Failed to notify host readiness");

        self.run_game_loop(&socket)        
    }

    fn bind_socket(&self) -> io::Result<UdpSocket>{
        let host_addr = format!("0.0.0.0:{HOST_PORT}");
        let socket = UdpSocket::bind(host_addr)?;
        println!("Host listening on port {HOST_PORT}...");

        Ok(socket)
    }

    fn run_game_loop(&mut self, socket: &UdpSocket) -> io::Result<()> {
        let tick_rate = Duration::from_secs_f64(1.0 / 2.0); // 2 fps for client update
        let mut next_tick = Instant::now();

        let mut buf = [0u8; 1024];
        socket.set_nonblocking(true)?;
        loop {
            let now = Instant::now();
            if now >= next_tick {
                // Drain incoming UDP packets
                loop {
                    match socket.recv_from(&mut buf) {
                        Ok((size, addr)) => {
                            let client_msg: ClientMessage = decode(&buf[..size]);
                            self.handle_message(client_msg, addr); 
                        },

                        Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                            break;
                        },

                        Err(e) => return Err(e),
                    }
                    
                }

                let (mut game_state,
                    mut player_updates,
                    mut player_logs
                ) = self.game.update();
                    
                for player_log in player_logs.drain(..) {
                    if player_log.player_id == ALL_PLAYER_ID {
                        self.logs.push(player_log.message);
                    } else {
                        self.player_logs
                            .entry(player_log.player_id)
                            .or_default()
                            .push(player_log.message);
                    }
                }

                if let GameState::Waiting{player_infos} = game_state {
                    let mut online_player_infos = Vec::new();
                    for mut player_info in player_infos {
                        if let Some(player_conn_ref) = self.connection_by_id
                            .get(&player_info.id) 
                        {
                            let player_conn = player_conn_ref.borrow();
                            let mut status = 0; // not ready
                            if player_conn.ready_to_start {
                                status = 1;
                            }
                            player_info.status = status;
                            online_player_infos.push(player_info);
                        } 
                    }

                    game_state = GameState::Waiting {
                        player_infos: online_player_infos
                    }
                }
                
                // send to all client (todo: use async)
                for player_conn_ref in self.connection_by_id.values() { 
                    let player_conn = player_conn_ref.borrow();

                    if let Some(index) = player_updates.iter()
                        .position(|x| x.player_id == player_conn.id)
                    {
                        // Game State
                        let mut clone_state = game_state.clone(); 

                        match &mut clone_state {
                            GameState::Waiting{player_infos} => {
                                Self::mask_player_ids(player_infos , player_conn.id);
                            },

                            GameState::Playing{
                                winners,
                                remaining_players,
                                playable_card_count: _,
                            } => {
                                Self::mask_player_ids(winners, player_conn.id);
                                Self::mask_player_ids(remaining_players, player_conn.id);
                            },

                            GameState::Finished{
                                winners,
                                remaining_secs: _,
                            } => {
                                Self::mask_player_ids(winners, player_conn.id);
                            },
                        }

                        // Player State
                        let player_state = player_updates.remove(index).state;
                        
                        // Player Log
                        let mut player_logs = self.logs.clone(); // todo : store timestamp and do ordering
                        if let Some(logs) = self.player_logs.remove(&player_conn.id){
                            player_logs.extend(logs);
                        }

                        let update_msg = HostMessage::Update {
                            game_state: clone_state, player_state, player_logs,
                        }; 
                        socket.send_to(&encode(&update_msg), player_conn.addr)?;
                    } else {
                        panic!("Player connection has no corresponding player update");
                    }
                }
                self.logs.clear();
                next_tick += tick_rate;
            }            
        }
    }

    fn handle_message(&mut self, message: ClientMessage, addr:SocketAddr) {
        match message {
            ClientMessage::Join {player_name} => {
                let is_success = self.player_join(&player_name, addr);
                if is_success{
                    self.logs.push(format!("{} has joined", player_name));
                }
            },

            ClientMessage::SetReady(is_ready) => {
                if let Some(player_conn_ref) = self.connection_by_addr.get(&addr) {
                    let mut player_conn = player_conn_ref.borrow_mut();
                    
                    player_conn.ready_to_start = is_ready;
                }
                let mut all_player_ready = true;
                for player_conn_ref in self.connection_by_addr.values(){
                    let player_conn = player_conn_ref.borrow();
                    if !player_conn.ready_to_start {
                        all_player_ready = false;
                        break;
                    }
                }

                if all_player_ready {
                    let mut player_ids : Vec<usize> = Vec::new();
                    for player_conn_ref in self.connection_by_addr.values(){
                        let mut player_conn = player_conn_ref.borrow_mut();
                        player_conn.ready_to_start = false;
                        player_ids.push(player_conn.id);
                    }

                    self.game.retain_players(&player_ids);

                    let max_player = 20;
                    self.game.fill_with_bots(max_player);

                    self.game.start();
                    self.logs.push("Game has started!".to_string());
                }
            },

           ClientMessage::Disconnect => {
                if let Some(player_conn_ref) = self.connection_by_addr.get(&addr) {
                    let player_conn = player_conn_ref.borrow(); 
                    self.game.remove_player(player_conn.id);
                    self.connection_by_id.remove(&player_conn.id);
                }
                
                self.connection_by_addr.remove(&addr);
            },

            ClientMessage::ReadyForNextDuel => {
                if let Some(player_conn_ref) = self.connection_by_addr.get(&addr) {
                    let player_conn = player_conn_ref.borrow(); 
                    self.game.finish_resting(player_conn.id);
                }
            },

            ClientMessage::ChooseCardToPlay(card) => {
                if let Some(player_conn_ref) = self.connection_by_addr.get(&addr) {
                    let player_conn = player_conn_ref.borrow();
                    player_conn.card_tx.send(card).expect("Failed to send player input card");
                }
            },
        }
    }

    fn player_join(&mut self, player_name: &str, addr: SocketAddr) -> bool {
        if self.connection_by_addr.contains_key(&addr) {
            return false;
        }

        let id = self.next_player_id;
        self.next_player_id += 1;
        let (card_tx, card_rx) = channel();

        // Create player connection
        let new_player_conn = Rc::new(RefCell::new(PlayerConnection {
            id, addr, card_tx, 
            ready_to_start: false,
        }));
        self.connection_by_id.insert(id, new_player_conn.clone());
        self.connection_by_addr.insert(addr, new_player_conn.clone());

        // Create player
        let online_strategy = OnlineStrategy {card_rx};
        let new_player = Player::new(id, player_name.to_string(), online_strategy);
        self.game.add_player(new_player);

        true
    }

    fn mask_player_ids(players: &mut [PlayerInfo], player_id: usize) {
        for player in players {
            player.id = if player.id == player_id {
                PLAYER_SELF
            } else {
                PLAYER_OTHER
            };
        }
    }
}

