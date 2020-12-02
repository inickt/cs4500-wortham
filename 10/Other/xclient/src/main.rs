use fish::client::proxy_client::ProxyClient;
use fish::client::strategy::ZigZagMinMaxStrategy;

use std::thread;
use std::time::Duration;

const TIMEOUT: Duration = Duration::from_secs(300);
const USAGE: &str = "usage: ./xclient <num_clients> <port> [ip_address]";

fn main() {
    let (num_clients, address) = parse_args();
    run_clients(num_clients, address);
}

fn run_clients(num_clients: usize, address: String) {
    let threads = (0..num_clients).map(|_| {
        let address = address.clone();
        thread::spawn(move || {
            let mut client = ProxyClient::new(Box::new(ZigZagMinMaxStrategy), &address, TIMEOUT)
                .expect("Unable to connect to server");
            client.tournament_loop();
        })
    }).collect::<Vec<_>>();
    for thread in threads {
        thread.join().ok();
    }
}

fn parse_args() -> (usize, String) {
    let args = std::env::args().collect::<Vec<_>>();
    let num_clients = args.get(1).and_then(|arg|  arg.parse().ok()).expect(USAGE);
    let port = args.get(2).expect(USAGE);
    let ip = args.get(3).map_or("127.0.0.1", String::as_str);
    (num_clients, format!("{}:{}", ip, port))
}
