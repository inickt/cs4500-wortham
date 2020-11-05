#![allow(non_camel_case_types)]
use std::collections::HashMap;

use serde_json::Deserializer;
use serde::{ Serialize, Deserialize };

use fish::common::action::Move;
use fish::common::board::Board;
use fish::common::direction::{ Direction, Direction::* };
use fish::common::gamestate::GameState;
use fish::common::penguin::PenguinId;
use fish::common::player::{ Player, PlayerId };
use fish::common::penguin::Penguin;

#[derive(Serialize, Deserialize)]
struct JSONPlayersAndBoard {
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

fn make_color_mapping(gamestate: &GameState, json_players: &[JSONPlayer]) -> HashMap<PlayerId, JSONColor> {
    gamestate.turn_order.iter().copied()
        .zip(json_players.iter().map(|json_player| json_player.color))
        .collect()
}

fn set_player_scores(gamestate: &mut GameState, json_players: &[JSONPlayer]) {
    assert_eq!(gamestate.turn_order.len(), json_players.len());

    for (turn, json_player) in gamestate.turn_order.iter().copied().zip(json_players.iter()) {
        let player = gamestate.players.get_mut(&turn).unwrap();
        player.score = json_player.score;
    }
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
            let penguin_id = penguin.penguin_id;

            // Must get the player again so that gamestate isn't mutably borrowed twice
            // during place_avatar_without_changing_turn
            let player = gamestate.players.get_mut(&player_id).unwrap();
            player.penguins.push(penguin);

            let tile_id = gamestate.board.get_tile(place[1], place[0]).unwrap().tile_id;
            gamestate.place_avatar_without_changing_turn(player_id, penguin_id, tile_id);
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
    let mut reachable_tiles = tile.all_reachable_tiles_in_direction(&gamestate.board, direction, &occupied_tiles);
    reachable_tiles.pop(); // Remove the current tile since it is considered reachable from itself in the helper above

    if reachable_tiles.is_empty() {
        false
    } else {
        let destination = reachable_tiles.last().unwrap().tile_id;
        let action = Move::new(penguin_id, destination);
        // unwrap the result just to assert success since we know the tile is reachable
        gamestate.move_avatar_for_current_player(action).unwrap();
        true
    }
}

fn serialize_board(board: &Board) -> JSONBoard {
    let mut rows = vec![];

    for row_i in 0 .. board.height {
        let mut new_row = vec![];
        for col_i in 0 .. board.width {
            let tile = board.get_tile(col_i, row_i);
            let fish_count = tile.map_or(0, |tile| tile.fish_count as u32);
            new_row.push(fish_count);
        }
        rows.push(new_row);
    }

    rows
}

fn serialize_player(player: &Player, board: &Board, color_mapping: &HashMap<PlayerId, JSONColor>) -> JSONPlayer {
    let mut places = vec![];

    for penguin in player.penguins.iter() {
        let tile_id = penguin.tile_id.unwrap(); // Input should contain only placed penguins, therefore all tile_ids should be Some(id)
        let position = board.get_tile_position(tile_id);
        places.push([position.y, position.x]);
    }

    JSONPlayer {
        color: color_mapping[&player.player_id],
        score: player.score,
        places
    }
}

fn serialize_players(gamestate: &GameState, color_mapping: &HashMap<PlayerId, JSONColor>) -> Vec<JSONPlayer> {
    let mut players_json = vec![];
    for id in gamestate.turn_order.iter() {
        let player = &gamestate.players[id];
        players_json.push(serialize_player(player, &gamestate.board, color_mapping));
    }

    players_json
}

fn serialize_gamestate(gamestate: &GameState, color_mapping: &HashMap<PlayerId, JSONColor>) -> JSONPlayersAndBoard {
    let board = serialize_board(&gamestate.board);
    let players = serialize_players(gamestate, color_mapping);

    JSONPlayersAndBoard { players, board }
}

fn main() {
    let stdin = std::io::stdin();
    let json = JSONPlayersAndBoard::from_reader(stdin.lock());
    let board = board_from_json(&json.board);

    let mut gamestate = GameState::new(board, json.players.len());

    // Our colors are different, so we have to map PlayerId -> JSONColor
    // for use while reserializing the GameState to make sure it has the right order.
    let player_color_mapping = make_color_mapping(&gamestate, &json.players);

    set_player_scores(&mut gamestate, &json.players);
    place_penguins(&mut gamestate, &json.players);

    let first_penguin = gamestate.current_player().penguins[0].penguin_id;

    for direction in &[North, Northeast, Southeast, South, Southwest, Northwest] {
        if try_move_penguin(&mut gamestate, first_penguin, *direction) {
            let state = serialize_gamestate(&gamestate, &player_color_mapping);
            print!("{}", serde_json::to_string(&state).unwrap());
            return;
        }
    }

    print!("False");
}
