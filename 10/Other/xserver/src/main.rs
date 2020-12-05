use fish::server::client::Client;
use fish::server::signup;
use fish::server::manager;
use fish::server::referee::ClientStatus;
use fish::common::board::Board;

use std::time::Duration;

// TODO move back to 1s
const REMOTE_CLIENT_TIMEOUT: Duration = Duration::from_secs(10);
const USAGE: &str = "usage: ./xserver <port>";

fn main() {
    let port = parse_args();
    run_tournament(port)
}

fn run_tournament(port: usize) {
    match signup::signup_clients(port, REMOTE_CLIENT_TIMEOUT) {
        Some(clients) => { 
            let boxed_clients = clients.into_iter().map(|c| Box::new(c) as Box<dyn Client>).collect();
            let board = Board::with_no_holes(5, 5, 2);
            let results = manager::run_tournament(boxed_clients, Some(board));

            let winners = results.iter().filter(|status| **status == ClientStatus::Won).count();
            let kicked = results.iter().filter(|status| **status == ClientStatus::Kicked).count();
            println!("[{},{}]", winners, kicked);
        },
        None => println!("Not enough players to start a tournament"),
    }
}

fn parse_args() -> usize {
    std::env::args().nth(1).expect(USAGE).parse().expect(USAGE)
}
