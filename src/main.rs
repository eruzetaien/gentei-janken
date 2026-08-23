use std::io;
use std::thread;

mod thread_pool;

// use thread_pool::ThreadPool;
// use gentei_janken::game;
use gentei_janken::network::Host;
use gentei_janken::network::Client;
// use gentei_janken::game::{
//     Player,
//     PlayStrategy,
//     RandomStrategy,
//     BalancedStrategy,
//     ProbabilityStrategy,
// };




fn main() {

    println!("Please input player name.");
    let mut player_name = String::new();

    io::stdin()
        .read_line(&mut player_name)
        .expect("Failed to read player name");

    let player_name = player_name.trim().to_string();

    println!("Host IP (leave empty to host):");
    let mut host_ip = String::new();

    io::stdin()
        .read_line(&mut host_ip)
        .expect("Failed to read host IP");

    let mut host_ip = host_ip.trim();

    // let pool = ThreadPool::new(4);
    if host_ip.is_empty() {
        println!("Starting as host...");
        let (ready_tx, ready_rx) = std::sync::mpsc::channel();

        thread::spawn(move || {
            Host::start(ready_tx).expect("Failed to run host");
        });

        // Wait until the host tells us it's ready
        ready_rx
            .recv()
            .expect("Host thread stopped before becoming ready");

        host_ip = "127.0.0.1";
    }     

    println!("Starting client...");

    Client::start(
        host_ip.parse().expect("Invalid host IP addres")
    ).expect("Failed to run client"); 
}


