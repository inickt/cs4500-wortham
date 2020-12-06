#![allow(non_camel_case_types)]
use serde_json::json;
use serde::{ Serialize, Deserialize };

use fish::common::board::Board;
use fish::common::gamestate::GameState;
use fish::common::penguin::Penguin;
use fish::common::game_tree::GameTree;

use fish::server::strategy;

#[derive(Serialize, Deserialize)]
struct JSONState {
    players: Vec<JSONPlayer>,
    board: JSONBoard,
}

type JSONBoard = Vec<Vec<u32>>;

#[derive(Serialize, Deserialize)]
struct JSONPlayer {
    color: JSONColor,
    score: usize, // do we need arbitrary precision? 4 says "Natural"
    places: Vec<JSONPosition>
}

#[derive(Copy, Clone, Serialize, Deserialize)]
enum JSONColor {
    red,
    white,
    brown,
    black,
}

/// Json pair of [ board_row, board_column ]
type JSONPosition = [u32; 2];


/// Converts a JSON representation of a board to
/// the board module's Board representation.
fn board_from_json(json_board: &JSONBoard) -> Board {
    let columns = json_board.iter().map(|row| row.len()).max().unwrap_or(0);
    let mut tiles = vec![];

    for json_row in json_board.iter() {
        let mut row = vec![];

        for &num_fish in json_row.iter() {
            row.push(num_fish);
        }

        // Boards may not contain an equal number of columns in each row,
        // push the remains of any smaller rows as holes
        for _ in json_row.len() .. columns {
            row.push(0);
        }

        tiles.push(row);
    }

    Board::from_tiles(tiles)
}

fn place_penguins(gamestate: &mut GameState, json_players: &[JSONPlayer]) {
    let player_ids = gamestate.turn_order.clone().into_iter();

    for (player_id, json_player) in player_ids.zip(json_players.iter()) {
        let player = gamestate.players.get_mut(&player_id).unwrap();
        player.penguins.clear();

        // Push a new penguin on each iteration, in case the given json_players
        // contains an uneven amount of penguins for each player.
        for place in json_player.places.iter() {
            let penguin = Penguin::new();

            // Must get the player again so that gamestate isn't mutably borrowed twice
            // during place_avatar_without_changing_turn
            let player = gamestate.players.get_mut(&player_id).unwrap();
            player.penguins.push(penguin);

            let tile_id = gamestate.board.get_tile(place[1], place[0]).unwrap().tile_id;
            gamestate.place_avatar_without_changing_turn(player_id, tile_id);
        }
    }
}

fn main() {
    let stdin = std::io::stdin();
    let json: serde_json::Value = serde_json::from_reader(stdin.lock()).unwrap();

    let depth: usize = serde_json::from_value(json[0].clone()).unwrap();
    let json_state: JSONState = serde_json::from_value(json[1].clone()).unwrap();

    let board = board_from_json(&json_state.board);
    let mut gamestate = GameState::new(board, json_state.players.len());
    place_penguins(&mut gamestate, &json_state.players);
    
    let mut game_tree = GameTree::new(&gamestate);

    // Serialization and setup is finished, find the minmax move and output it if found,
    // or false otherwise.
    if game_tree.is_game_over() {
        println!("false");
    } else {
        let move_ = strategy::find_minmax_move(&mut game_tree, depth);
        let from_pos = gamestate.board.get_tile_position(move_.from);
        let to_pos = gamestate.board.get_tile_position(move_.to);
        println!("{}", json!([[from_pos.y, from_pos.x], [to_pos.y, to_pos.x]]));
    }
}
