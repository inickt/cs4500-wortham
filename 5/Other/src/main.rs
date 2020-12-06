#![allow(non_camel_case_types)]
use serde_json::{json, Deserializer};
use serde::{ Serialize, Deserialize };

use fish::common::action::Move;
use fish::common::board::Board;
use fish::common::direction::Direction::*;
use fish::common::gamestate::GameState;
use fish::common::penguin::Penguin;
use fish::common::tile::TileId;
use fish::common::game_tree::GameTree;
use fish::common::util::all_min_by_key;

#[derive(Serialize, Deserialize)]
struct JSONStateAndMove {
    state: JSONState,
    from: JSONPosition,
    to: JSONPosition,
}

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


impl JSONStateAndMove {
    /// Deserializes a JSON string into a JSONPlayerAndBoard
    /// Assumes the reader will contain only valid JSON
    pub fn from_reader<R: std::io::Read>(reader: R) -> Self {
        let mut de = Deserializer::from_reader(reader);
        Self::deserialize(&mut de).ok().unwrap()
    }
}

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

fn move_from_json(state: &GameState, from_pos: JSONPosition, to_pos: JSONPosition) -> Move {
    // tile should be valid by constraint of spec
    let from = state.board.get_tile_id(from_pos[1], from_pos[0]).unwrap();
    let to = state.board.get_tile_id(to_pos[1], to_pos[0]).unwrap();
    Move { from, to }
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

/// Finds a move to a neighbor of first_player_tile
/// in the order North, Northeast, Southeast, South, Southwest, Northwest
/// and breaks ties by moving the penguin with the lowest row and column, in that order
fn find_neighboring_move(game: &GameTree, first_player_tile: TileId) -> Option<Move> {
    let state = game.get_state();
    let moves = match game {
        GameTree::Turn { valid_moves, .. } => valid_moves,
        _ => return None,
    };

    let moves = moves.into_iter().map(|(move_, _)| *move_);

    let player_tile = state.board.tiles.get(&first_player_tile).unwrap();
    for direction in &[North, Northeast, Southeast, South, Southwest, Northwest] {
        match player_tile.get_neighbor_id(*direction) {
            Some(neighbor) => {
                let moves = moves.clone().filter(|move_| move_.to == *neighbor);
                let mut moves = all_min_by_key(moves, |move_| state.board.get_tile_position(move_.from));
                match moves.nth(0) {
                    Some(move_) => return Some(move_),
                    None => (), // continue loop, check next direction
                }
            }
            None => (),
        }
    }

    None
}

fn main() {
    let stdin = std::io::stdin();
    let json = JSONStateAndMove::from_reader(stdin.lock());
    let board = board_from_json(&json.state.board);

    let mut gamestate = GameState::new(board, json.state.players.len());

    place_penguins(&mut gamestate, &json.state.players);

    let second_player = gamestate.turn_order[1];
    
    let mut game_tree = GameTree::new(&gamestate);
    let move_ = move_from_json(&gamestate, json.from, json.to);

    let tree_after_move = game_tree.get_game_after_move(move_).unwrap();

    let gamestate = tree_after_move.get_state();

    if gamestate.current_turn != second_player {
        // second player was skipped in turn order, they had no moves
        print!("false")
    } else {
        match find_neighboring_move(&tree_after_move, move_.to) {
            Some(move_) => {
                let from_pos = gamestate.board.get_tile_position(move_.from);
                let to_pos = gamestate.board.get_tile_position(move_.to);
                print!("{}", json!([[from_pos.y, from_pos.x], [to_pos.y, to_pos.x]]));
            },
            None => print!("false"),
        }
    }
}
