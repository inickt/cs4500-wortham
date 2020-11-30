use crate::common::gamestate::GameState;
use crate::common::action::PlayerMove;
use crate::common::board::Board;
use crate::common::player::{ Player, PlayerColor };
use crate::common::util;

use serde::{ Serialize, Deserialize };
use serde_json::json;

#[derive(Deserialize)]
pub struct Message<T, U> {
    pub name: String,
    pub arguments: (T, U),
}

#[derive(Serialize, Deserialize)]
pub struct JSONGameState {
    pub players: Vec<JSONPlayer>,
    pub board: JSONBoard,
}

type JSONBoard = Vec<Vec<u32>>;

#[derive(Serialize, Deserialize)]
struct JSONPlayer {
    pub color: PlayerColor,
    pub score: usize, // do we need arbitrary precision? 4 says "Natural"
    pub places: Vec<JSONPosition>
}

/// Json pair of [ board_row, board_column ]
type JSONPosition = [u32; 2];

/// Json pair of [ from-pos, to-pos ]
type JSONAction = [JSONPosition; 2];


// All types of server->client messages:
pub type Start = Message<bool, ()>;
pub type PlayingAs = Message<PlayerColor, ()>;
pub type PlayingWith = Message<Vec<PlayerColor>, ()>;
pub type Setup = Message<JSONGameState, ()>;
pub type TakeTurn = Message<JSONGameState, Vec<JSONAction>>;
pub type End = Message<bool, ()>;

/// Return a start message encoded in json in a String
pub fn start_message() -> String {
    serde_json::to_string(&json!([ "start", [true] ])).unwrap()
}

pub fn playing_as_message(color: PlayerColor) -> String {
    serde_json::to_string(&json!([ "playing-as", [color] ])).unwrap()
}

pub fn playing_with_message(opponents: &[PlayerColor]) -> String {
    serde_json::to_string(&json!([ "playing-with", [opponents] ])).unwrap()
}

pub fn setup_message(state: &GameState) -> String {
    let state = serialize_gamestate(state);
    serde_json::to_string(&json!([ "setup", [state] ])).unwrap()
}

pub fn take_turn_message(state: &GameState, moves: &[PlayerMove]) -> String {
    let state = serialize_gamestate(state);
    let actions = convert_to_json_actions(moves);
    serde_json::to_string(&json!([ "take-turn", [ state, actions ] ])).unwrap()
}

pub fn end_message(winner: bool) -> String {
    serde_json::to_string(&json!([ "end", [ winner ] ])).unwrap()
}


fn convert_to_json_actions(moves: &[PlayerMove]) -> Vec<JSONAction> {
    util::map_slice(moves, |move_| [ [move_.from.y, move_.from.x] , [move_.to.y, move_.to.x] ])
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

fn serialize_player(player: &Player, board: &Board) -> JSONPlayer {
    let places = player.penguins.iter().filter_map(|penguin| {
        let tile_id = penguin.tile_id?;
        let position = board.get_tile_position(tile_id);
        Some([position.y, position.x])
    }).collect();

    JSONPlayer {
        color: player.color,
        score: player.score,
        places
    }
}

fn serialize_players(gamestate: &GameState) -> Vec<JSONPlayer> {
    util::map_slice(&gamestate.turn_order, |id| {
        serialize_player(&gamestate.players[id], &gamestate.board)
    })
}

fn serialize_gamestate(gamestate: &GameState) -> JSONGameState {
    let board = serialize_board(&gamestate.board);
    let players = serialize_players(gamestate);

    JSONGameState { players, board }
}
