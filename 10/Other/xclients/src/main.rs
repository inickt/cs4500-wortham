use fish::client::client_to_server_proxy::ClientToServerProxy;
use fish::common::action::{ Placement, Move };
use fish::common::gamestate::GameState;
use fish::common::game_tree::GameTree;
use fish::server::ai_client::AIClient;
use fish::server::strategy;

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
            let ai_player = AIClient::new(Box::new(ClientStrategy));
            let mut client = ClientToServerProxy::new("AIClient".to_string(), Box::new(ai_player), &address, TIMEOUT)
                .expect(&format!("Unable to connect to server on thread {}", num));

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

struct ClientStrategy;
impl strategy::Strategy for ClientStrategy {
    fn find_placement(&mut self, gamestate: &GameState) -> Placement {
        strategy::find_zigzag_placement(gamestate)
    }

    fn find_move(&mut self, game: &mut GameTree) -> Move {
        strategy::find_minmax_move(game, 1)
    }
}
