use fish::common::tile::TileId;
use fish::common::gamestate::GameState;
use fish::common::action::{ Placement, Move, PlayerMove };
use fish::common::util::make_n;
use fish::common::player::PlayerColor;
use fish::server::ai_client::AIClient;
use fish::server::referee::run_game;
use fish::server::client::Client;
use fish::server::remote_client::RemoteClient;
use fish::client::client_to_server_proxy::ClientToServerProxy;
use fish::common::util::try_with_timeout;

use std::net::{ TcpListener, TcpStream };
use std::time::{ Duration, Instant };
use std::thread;
use std::io::Write;

const USAGE: &str = "Usage: ./xgui <AI player count (1-3)>";
const ADDRESS: &str = "127.0.0.1:8080";
const TIMEOUT_SECS: u64 = 1;
const PLAYER_USAGE: &str = "Usage:\n'place [tile_id]'\n'move [penguin_id] to [tile_id]'";

fn main() {
    start_game_server();

    let timeout = Duration::from_secs(TIMEOUT_SECS);

    match try_with_timeout(timeout, || {
        let client = Box::new(HumanClient);
        ClientToServerProxy::new("Human".to_string(), client, ADDRESS, Duration::from_secs(30))
    }) {
        // run a human controlled player loop
        Some(mut proxy) => { proxy.tournament_loop(); },
        None => eprintln!("Unable to connect to stream while creating proxy"),
    }
}

fn start_game_server() {
    // create a tournament with n - 1 ai players and 1 human controlled player
    thread::spawn(|| {
        let listener = TcpListener::bind(ADDRESS).unwrap();
        listener.set_nonblocking(true).ok();

        let players = create_players(&listener);
        let game_result = run_game(players, None);
        let player_result = game_result.final_statuses.last().unwrap();
        println!("END GAME STATE:\n{:?}\nFINAL PLAYER STATUS: {:?}", game_result.final_state, player_result);
    });
}

struct HumanClient;

impl Client for HumanClient {
    fn tournament_starting(&mut self) -> Option<()> {
        println!("Tournament Starting!");
        Some(())
    }

    fn tournament_ending(&mut self, won: bool) -> Option<()> {
        println!("Tournament Ending!\nYou {}!", if won { "won" } else { "lost" });
        Some(())
    }

    fn initialize_game(&mut self, _initial_gamestate: &GameState, player_color: PlayerColor) -> Option<()> {
        println!("Starting new game, you are {:?}", player_color);
        Some(())
    }

    fn get_placement(&mut self, gamestate: &GameState) -> Option<Placement> {
        println!("{:?}\nYour turn to place a penguin:\n", gamestate);
        Some(parse_placement_input())
    }

    fn get_move(&mut self, gamestate: &GameState, _previous: &[PlayerMove]) -> Option<Move> {
        println!("{:?}\nYour turn to make a move:\n", gamestate);
        Some(parse_move_input())
    }
}

fn parse_placement_input() -> Placement {
    print!("{}\n> ", PLAYER_USAGE);
    std::io::stdout().flush().ok();
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
    let words = input.trim().split(" ").collect::<Vec<_>>();

    match (words[0], words[1].parse()) {
        ("place", Ok(id)) => Placement::new(TileId(id)),
        _ => parse_placement_input(),
    }
}

fn parse_move_input() -> Move {
    print!("{}\n> ", PLAYER_USAGE);
    std::io::stdout().flush().ok();
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
    let words = input.trim().split(" ").collect::<Vec<_>>();

    match (words[0], words[1].parse(), words[2], words[3].parse()) {
        ("move", Ok(from_tile), "to", Ok(to_tile)) => 
            Move::new(TileId(from_tile), TileId(to_tile)),
        _ => parse_move_input(),
    }
}

fn wait_for_connection(listener: &TcpListener, start_time: Instant) -> TcpStream {
    match listener.incoming().next() {
        Some(Ok(stream)) => stream,
        _ if start_time.elapsed().as_secs() < TIMEOUT_SECS => wait_for_connection(listener, start_time),
        _ => unreachable!("Could not connect to localhost: server thread should have started connection")
    }
}

fn get_ai_player_count() -> usize {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    if args.is_empty() {
        panic!(USAGE);
    }
    let count = args[0].parse().expect(USAGE);
    assert!(count >= 1 && count <= 3, USAGE);
    count
}

fn create_players(listener: &TcpListener) -> Vec<Box<dyn Client>> {
    let mut players: Vec<_> = make_n(get_ai_player_count(),
        |_| Box::new(AIClient::with_zigzag_minmax_strategy()) as Box<dyn Client>);

    let stream = wait_for_connection(listener, Instant::now());
    players.push(Box::new(RemoteClient::new(stream, Duration::from_secs(30))));
    players
}
