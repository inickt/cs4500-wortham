use fish::server::signup;
use fish::server::manager;
use fish::server::referee::ClientStatus;
use fish::common::board::Board;

use std::time::Duration;

const REMOTE_CLIENT_TIMEOUT: Duration = Duration::from_secs(1);

fn main() {
    let port = match std::env::args().nth(1) {
        Some(port) => port,
        None => panic!("usage: ./xserver <port>"),
    };
    run_tournament(port)
}

fn run_tournament(port: usize) {
    match signup::signup_clients(&port, REMOTE_CLIENT_TIMEOUT) {
        Some(clients) => { 
            let board = Board::with_no_holes(5, 5, 2);
            let results = manager::run_tournament(clients, Some(board));
            
            let winners = results.iter().filter(|status| **status == ClientStatus::Won).count();
            let kicked = results.iter().filter(|status| **status == ClientStatus::Kicked).count();
            println!("[{},{}]", winners, kicked);
        },
        None => println!("Not enough players to start a tournament"),
    }
}
