use std::io;
use std::thread;
use std::panic;

use crossterm::{
    cursor,
    execute,
    terminal::{disable_raw_mode, Clear, ClearType},
};

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




fn main() -> io::Result<()> {
    install_panic_hook();
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
            let mut player_host = Host::new();
            player_host.start(ready_tx).expect("Failed to run host");
        });

        // Wait until the host tells us it's ready
        ready_rx
            .recv()
            .expect("Host thread stopped before becoming ready");

        host_ip = "127.0.0.1";
    }     

    println!("Starting client...");

    let mut player_client = Client::new()?;
    player_client.start(
        host_ip.parse().expect("Invalid host IP addres"),
        player_name,

    ).expect("Failed to run client");

    Ok(())
}

pub fn install_panic_hook() {
    let original_hook = panic::take_hook();
    panic::set_hook(Box::new(move |info| {
        let _ = disable_raw_mode();
        let _ = execute!(
            io::stdout(),
            cursor::Show,
            Clear(ClearType::All),
        );
        original_hook(info);
    }));
}
