use crate::common::gamestate::GameState;
use crate::common::action::PlayerMove;
use crate::common::board::Board;
use crate::common::player::{ Player, PlayerColor };
use crate::common::penguin::Penguin;
use crate::common::util;

use serde::{ Serialize, Deserialize };
use serde_json::json;

#[derive(Serialize, Deserialize)]
pub struct JSONGameState {
    pub players: Vec<JSONPlayer>,
    pub board: JSONBoard,
}

type JSONBoard = Vec<Vec<u32>>;

#[derive(Serialize, Deserialize)]
pub struct JSONPlayer {
    pub color: PlayerColor,
    pub score: usize, // do we need arbitrary precision? 4 says "Natural"
    pub places: Vec<JSONPosition>
}

/// Json pair of [ board_row, board_column ]
type JSONPosition = [u32; 2];

/// Json pair of [ from-pos, to-pos ]
type JSONAction = [JSONPosition; 2];


/// All the types of client-server messages.
///
/// This type is intended for deserializing messages
/// of the format [ "variant-name", [ ... ] ] where the
/// ... contains the arguments of the message.
///
/// Most of these variants contain a single element tuple (T,)
/// so that deserializing them from a 1-element array works.
#[derive(Deserialize)]
#[serde(tag = "name", content = "arguments")]
#[serde(rename_all = "kebab-case")]
pub enum ServerToClientMessage {
    Start((bool,)),
    PlayingAs((PlayerColor,)),
    PlayingWith((Vec<PlayerColor>,)),
    Setup((JSONGameState,)),
    TakeTurn(JSONGameState, Vec<JSONAction>),
    End((bool,)),
}

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

impl JSONGameState {
    pub fn to_common_game_state(self) -> GameState {
        let board = Board::from_tiles(self.board);

        let mut gamestate = GameState::new(board, self.players.len());

        set_player_scores(&mut gamestate, &self.players);
        place_penguins(&mut gamestate, &self.players);

        gamestate
    }
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

        // Push a new penguin on each iteration, in case the given json_players
        // contains an uneven amount of penguins for each player.
        for (penguin, place) in player.penguins.iter_mut().zip(json_player.places.iter()) {
            // instead of using .map, the tile id here is unwrapped then rewrapped in Some
            // to force an assertion that the json should never have invalid penguin placements.
            penguin.tile_id = Some(gamestate.board.get_tile(place[1], place[0]).unwrap().tile_id);
        }
    }
}
