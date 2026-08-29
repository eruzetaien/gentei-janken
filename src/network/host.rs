use std::collections::HashMap;
use std::io;
use std::net::{
    IpAddr,
    SocketAddr,
    UdpSocket,
};
use std::sync::mpsc::{Receiver, Sender, channel};
use std::time::{Duration, Instant};

use crate::player_loader::load_players_from_json;
use crate::constant::HOST_PORT;
use crate::game::{
    Game, GameState,
    Player, OnlineStrategy
};
use crate::network::{
    PlayerConnection,
    HostMessage, ClientMessage,
    encode, decode
};

pub struct Host {
    game: Game,
    player_connections: HashMap<usize, PlayerConnection>,
    next_player_id: usize,
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
            player_connections: HashMap::new(),
            next_player_id: 0,
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
            
            let now = Instant::now();
            if now >= next_tick {
                for player_conn in self.player_connections.values() {
                    let game_state = self.game.game_state();
                    let update_msg = HostMessage::Update {game_state}; 
                    socket.send_to(&encode(&update_msg), player_conn.addr)?;
                }

                next_tick += tick_rate;
            }            
        }
    }

    fn handle_message(&mut self, message: ClientMessage, addr:SocketAddr) {
        match message {
            ClientMessage::Join {player_name} => {
                self.player_join(player_name, addr);
            }
        }
    }

    fn player_join(&mut self, player_name: String, addr: SocketAddr) {
        for player_conn in self.player_connections.values() {
            if player_conn.addr == addr {return;}
        }

        println!("{:?} join", player_name);
        let id = self.next_player_id;
        self.next_player_id += 1;
        let (card_tx, card_rx) = channel();

        // Create player connection
        let new_player_conn = PlayerConnection {id, addr, card_tx};
        self.player_connections.insert(id, new_player_conn);

        // Create player
        let online_strategy = OnlineStrategy {card_rx};
        let new_player = Player::new(id, player_name, online_strategy);
        self.game.add_player(new_player);

    }
}

