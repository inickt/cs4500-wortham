use std::collections::HashSet;

use serde_json::Deserializer;
use serde::{ Serialize, Deserialize };

use fish::common::board::Board;
use fish::common::boardposn::BoardPosn;
use fish::common::direction::{ Direction, Direction::* };
use fish::common::gamestate::GameState;
use fish::common::penguin::PenguinId;

#[derive(Serialize, Deserialize)]
struct JSONPlayersAndBoard {
    players: Vec<JSONPlayer>,
    board: Vec<Vec<u32>>, // vector of rows
}

#[derive(Serialize, Deserialize)]
struct JSONPlayer {
    _color: JSONColor,
    _score: usize, // do we need arbitrary precision? 4 says "Natural"
    places: Vec<JSONPosition>
}

#[derive(Serialize, Deserialize)]
enum JSONColor {
    Red,
    White,
    Brown,
    Black,
}

/// Json pair of [ board_row, board_column ]
type JSONPosition = [u32; 2];


impl JSONPlayersAndBoard {
    /// Deserializes a JSON string into a JSONPlayerAndBoard
    /// Assumes the reader will contain only valid JSON
    pub fn from_reader<R: std::io::Read>(reader: R) -> Self {
        let mut de = Deserializer::from_reader(reader);
        Self::deserialize(&mut de).ok().unwrap()
    }
}

/// Converts a JSON representation of a board to
/// the board module's Board representation.
fn board_from_json(board: &[Vec<u32>]) -> Board {
    let rows = board.len();
    let columns = board.iter().map(|row| row.len()).max().unwrap_or(0);
    let mut holes = Vec::new();

    for (row_i, row) in board.iter().enumerate() {
        for (col_i, &num_fish) in row.iter().enumerate() {
            if num_fish == 0 {
                holes.push(BoardPosn::from((col_i as u32, row_i as u32)));
            }
        }

        // Boards may not contain an equal number of columns in each row,
        // push the remains of any smaller rows as holes
        for col_i in row.len() .. columns {
            holes.push(BoardPosn::from((col_i as u32, row_i as u32)));
        }
    }

    Board::with_holes(rows as u32, columns as u32, holes, 0)
}

fn place_penguins(gamestate: &mut GameState, json_players: &[JSONPlayer]) {
    let all_penguins: Vec<Vec<_>> =
        gamestate.players.iter().map(|(&player_id, player)| {
            player.penguins.iter().map(move |penguin| (player_id, penguin.penguin_id)).collect()
        }).collect();

    // This just completely ignores the player colors and just assigns each player their
    // penguins in player order.
    for (penguin_ids, json_player) in all_penguins.into_iter().zip(json_players.iter()) {
        assert_eq!(penguin_ids.len(), json_player.places.len());
        for ((player_id, penguin_id), place) in penguin_ids.into_iter().zip(json_player.places.iter()) {
            let tile_id = gamestate.board.get_tile(place[0], place[1]).unwrap().tile_id;
            gamestate.place_avatar_for_player(player_id, penguin_id, tile_id);
        }
    }
}

/// Try to move a penguin to the first tile in the given direction.
/// Moves the penguin (without changing the current turn) and returns true on success.
/// Returns false on failure.
fn try_move_penguin(gamestate: &mut GameState, penguin_id: PenguinId, direction: Direction) -> bool {
    let penguin = gamestate.current_player().find_penguin(penguin_id).unwrap();
    let tile = gamestate.get_tile(penguin.tile_id.unwrap()).unwrap();
    let occupied_tiles = gamestate.get_occupied_tiles();
    let reachable_tiles = tile.all_reachable_tiles_in_direction(&gamestate.board, direction, &occupied_tiles);
    if reachable_tiles.is_empty() {
        false
    } else {
        let destination = reachable_tiles.last().unwrap().tile_id;
        // unwrap the result just to assert success since we know the tile is reachable
        gamestate.move_avatar_for_player(gamestate.current_turn, penguin_id, destination).unwrap();
        true
    }
}

fn serialize_gamestate(gamestate: &GameState) -> JSONPlayersAndBoard {
    unimplemented!()
}

fn main() {
    let stdin = std::io::stdin();
    let json = JSONPlayersAndBoard::from_reader(stdin.lock());
    let board = board_from_json(&json.board);

    let gamestate = GameState::new(0, board, json.players.len());
    let mut gamestate = gamestate.borrow_mut();

    place_penguins(&mut gamestate, &json.players);

    let first_penguin = gamestate.current_player().penguins[0].penguin_id;

    for direction in &[North, Northeast, Southeast, South, Southwest, Northwest] {
        if try_move_penguin(&mut gamestate, first_penguin, *direction) {
            let state = serialize_gamestate(&gamestate);
            println!("{}", serde_json::to_string(&state).unwrap());
        }
    }

    println!("False");
}
