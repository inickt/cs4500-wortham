use fish::client::client_to_server_proxy::ClientToServerProxy;
use fish::server::ai_client::AIClient;
use fish::server::strategy::ZigZagMinMaxStrategy;

use std::thread;
use std::time::Duration;

const TIMEOUT: Duration = Duration::from_secs(300);
const USAGE: &str = "usage: ./xclient <num_clients> <port> [ip_address]";

fn main() {
    let (num_clients, address) = parse_args();
    run_clients(num_clients, address);
}

fn run_clients(num_clients: usize, address: String) {
    let threads = (0..num_clients).map(|num| {
        let address = address.clone();
        thread::spawn(move || {
            let ai_player = AIClient::new(Box::new(ZigZagMinMaxStrategy));
            let mut client = ClientToServerProxy::new("AIClient", ai_player, &address, TIMEOUT)
                .expect(format!("Unable to connect to server on thread {}", num));
            match client.tournament_loop() {
                Some(won) => println!("AI Player {} completed tournament and {}.", num, if won { "won" } else { "lost" }),
                None =>  println!("AI Player {} was kicked or another error occurred.", num),
            };
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
