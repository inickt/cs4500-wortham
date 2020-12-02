use fish::client::proxy_client::ProxyClient;
use fish::client::strategy::ZigZagMinMaxStrategy;

use std::thread;
use std::time::Duration;

// TODO maybe should be longer? doesn't hurt to wait a while
const TIMEOUT: Duration = Duration::from_secs(30);

fn main() {
    let (num_clients, address) = parse_args();
    run_clients(num_clients, address);
}

fn run_clients(num_clients: usize, address: &str) {
    for _ in 0..num_clients {
        thread::spawn(|| {
            // TODO should probably close whole program if any can't connect?
            let client = ProxyClient::new(Box::new(ZigZagMinMaxStrategy), address, TIMEOUT).expect("Unable to connect to server");
            client.tournament_loop();
        });
    }
}

fn parse_args() -> (usize, &str) {
    // TODO fix this
    let num_clients = match std::env::args().nth(1).parse() {
        Ok(num_clients) => num_clients,
        Err(_) => panic!("usage: ./xserver <port>"),
    };

    let port = match std::env::args().nth(1) {
        Some(port) => port,
        None => panic!("usage: ./xserver <port>"),
    };

    (num_clients, "")
}