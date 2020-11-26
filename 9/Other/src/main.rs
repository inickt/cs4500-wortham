use fish::common::tile::TileId;
use fish::common::gamestate::GameState;
use fish::common::action::{ Action, Placement, Move };
use fish::common::util::make_n;
use fish::common::penguin::PenguinId;
use fish::common::player::PlayerId;
use fish::client::player::InHousePlayer;
use fish::server::referee::run_game;
use fish::server::serverclient::ClientProxy;
use fish::server::connection::PlayerConnection;

use std::net::{ TcpStream, TcpListener };
use std::time::{ Duration, Instant };
use std::thread;
use std::io::{ Read, Write };

use serde::{ Serialize, Deserialize };
use serde_json::Deserializer;

const USAGE: &str = "Usage: ./xgui <AI player count (1-3)>";
const ADDRESS: &str = "127.0.0.1:8080";
const TIMEOUT_SECS: u64 = 1;
const PLAYER_USAGE: &str = "Usage:\n'place [tile_id]'\n'move [penguin_id] to [tile_id]'";

fn main() {
    // create a tournament with n - 1 ai players and 1 human controlled player
    thread::spawn(|| {
        let players = create_players();
        let game_result = run_game(players, None);
        let player_result = game_result.final_statuses.last().unwrap();
        thread::sleep(Duration::from_secs(1));
        println!("END GAME STATE:\n{:?}\nFINAL PLAYER STATUS:{:?}", game_result.final_state, player_result);
    });

    // run a human controlled player loop
    let human_player = PlayerId(get_ai_player_count());
    let mut stream = wait_for_connection(Instant::now());
    let mut deserializer = Deserializer::from_reader(stream.try_clone().unwrap());
    loop {
        match GameState::deserialize(&mut deserializer) {
            Ok(game_state) if !game_state.is_game_over() => {
                println!("{:?}", game_state);
                if game_state.current_turn == human_player {
                    let action = get_player_input();
                    let serialized = serde_json::to_string(&action).unwrap();
                    stream.write(serialized.as_bytes()).unwrap();
                }
            },
            Ok(_) => break,
            Err(err) if err.is_eof() => continue,
            _ => panic!(),
        }
    }
}

// human player input loop, allows for:
// - 'place [tile_id]'
// - 'move [penguin_id] to [tile_id]'
fn get_player_input() -> Action {
    print!("{}\n> ", PLAYER_USAGE);
    std::io::stdout().flush().ok();
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
    let words = input.trim().split(" ").collect::<Vec<_>>();
    if words.len() == 2 {
        parse_placement(words)
    } else if words.len() == 4 {
        parse_move(words)
    } else {
        get_player_input()
    }
}

fn parse_placement(words: Vec<&str>) -> Action {
    match (words[0], words[1].parse()) {
        ("place", Ok(id)) => Action::PlacePenguin(Placement::new(TileId(id))),
        _ => get_player_input(),
    }
}

fn parse_move(words: Vec<&str>) -> Action {
    match (words[0], words[1].parse(), words[2], words[3].parse()) {
        ("move", Ok(penguin_id), "to", Ok(tile_id)) => 
            Action::MovePenguin(Move::new(PenguinId(penguin_id), TileId(tile_id))),
        _ => get_player_input(),
    }
}

fn wait_for_connection(start_time: Instant) -> TcpStream {
    match TcpStream::connect(ADDRESS) {
        Ok(stream) => stream,
        Err(_) if start_time.elapsed().as_secs() < TIMEOUT_SECS => wait_for_connection(start_time),
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

fn create_players() -> Vec<ClientProxy> {
    let mut players: Vec<_> = make_n(get_ai_player_count(),
        |_| ClientProxy::InHouseAI(InHousePlayer::with_zigzag_minmax_strategy()));
    let host = TcpListener::bind(ADDRESS).expect("Could not connect to localhost");
    players.push(ClientProxy::Remote(PlayerConnection::new(host)));
    players
}
