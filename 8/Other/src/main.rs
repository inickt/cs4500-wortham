use serde_json::json;
use serde::{ Serialize, Deserialize };

use fish::common::board::Board;
use fish::common::gamestate::GameState;
use fish::common::game_tree::GameTree;
use fish::common::action::{Move, Placement};
use fish::server::referee;
use fish::server::client::Client;
use fish::server::strategy;
use fish::server::ai_client::AIClient;


#[derive(Serialize, Deserialize, Debug)]
struct GameDescription {
    pub row: u32,     // in [2, 5]
    pub column: u32,  // in [2, 5]
    pub players: Vec<JSONPlayer>,
    pub fish: usize,    // in [1,5]
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct JSONPlayer {
    pub name: String,
    pub depth: usize
}

impl strategy::Strategy for JSONPlayer {
    fn find_placement(&mut self, gamestate: &GameState) -> Placement {
        strategy::find_zigzag_placement(gamestate)
    }

    fn find_move(&mut self, game: &mut GameTree) -> Move {
        strategy::find_minmax_move(game, self.depth)
    }
}

fn main() {
    let stdin = std::io::stdin();
    let description: GameDescription = serde_json::from_reader(stdin.lock()).unwrap();

    let board = Board::with_no_holes(description.row, description.column, description.fish);

    let players = description.players.iter().map(|player| {
        Box::new(AIClient::new(Box::new(player.clone()))) as Box<dyn Client>
    }).collect();

    // PlayerResult = Won | Lost | Kicked
    let player_results = referee::run_game(players, Some(board)).final_statuses;

    let mut winning_players = player_results.iter().zip(description.players.iter())
        .filter(|(result, _)| **result == referee::ClientStatus::Won)
        .map(|(_, desc)| &desc.name)
        .collect::<Vec<_>>();
        
    winning_players.sort();
    println!("{}", json!(winning_players));
}
