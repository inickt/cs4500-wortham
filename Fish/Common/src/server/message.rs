use crate::common::gamestate::GameState;
use crate::common::action::{ PlayerMove, Placement, Move };
use crate::common::board::Board;
use crate::common::player::{ Player, PlayerColor };
use crate::common::penguin::Penguin;
use crate::common::util;

use serde::{ Serialize, Deserialize };
use serde_json::json;

#[derive(Serialize, Deserialize, Debug)]
pub struct JSONGameState {
    pub players: Vec<JSONPlayer>,
    pub board: JSONBoard,
}

type JSONBoard = Vec<Vec<u32>>;

#[derive(Serialize, Deserialize, Debug)]
pub struct JSONPlayer {
    pub color: PlayerColor,
    pub score: usize, // do we need arbitrary precision? 4 says "Natural"
    pub places: Vec<JSONPosition>
}

/// Json pair of [ board_row, board_column ]
type JSONPosition = [u32; 2];

pub fn placement_to_json_position(board: &Board, placement: Placement) -> JSONPosition {
    let board_position = board.get_tile_position(placement.tile_id);
    [board_position.y, board_position.x]
}

/// Json pair of [ from-pos, to-pos ]
type JSONAction = [JSONPosition; 2];

pub fn move_to_json_action(board: &Board, move_: Move) -> JSONAction {
    let from_position = board.get_tile_position(move_.from);
    let to_position = board.get_tile_position(move_.to);
    [ [from_position.y, from_position.x], [to_position.y, to_position.x] ]
}

/// All the types of client-server messages.
///
/// This type is intended for deserializing messages
/// of the format [ "variant-name", [ ... ] ] where the
/// ... contains the arguments of the message.
///
/// Most of these variants contain a single element tuple (T,)
/// so that deserializing them from a 1-element array works.
#[derive(Deserialize, Debug)]
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

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub enum JSONVoid {
    Void
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum ClientToServerMessage {
    Void(JSONVoid),
    Position(JSONPosition),
    Action(JSONAction), 
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


pub fn convert_to_json_actions(moves: &[PlayerMove]) -> Vec<JSONAction> {
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
    let mut json_players = util::map_slice(&gamestate.turn_order, |id| {
        serialize_player(&gamestate.players[id], &gamestate.board)
    });
    // current player should be first
    let current_turn_index = gamestate.players.iter().position(|player| {
        *player.0 == gamestate.current_turn
    }).unwrap();

    json_players.rotate_left(current_turn_index);
    json_players
}

pub fn serialize_gamestate(gamestate: &GameState) -> JSONGameState {
    let board = serialize_board(&gamestate.board);
    let players = serialize_players(gamestate);

    JSONGameState { players, board }
}

impl JSONGameState {
    pub fn to_common_game_state(self, player_count: usize) -> GameState {
        let board = Board::from_tiles(self.board);

        // Use the passed-in original player count rather than self.players.len()
        // in case some players have been kicked, so that we can still give the
        // correct penguin count to each player.
        let mut gamestate = GameState::new(board, player_count);

        remove_kicked_players(&mut gamestate, &self.players);
        set_current_turn(&mut gamestate, &self.players);
        set_player_scores(&mut gamestate, &self.players);
        place_penguins(&mut gamestate, &self.players);

        gamestate
    }
}

fn set_current_turn(gamestate: &mut GameState, json_players: &[JSONPlayer]) {
    if !json_players.is_empty() {
        let player = gamestate.get_player_by_color_mut(json_players[0].color);
        gamestate.current_turn = player.unwrap().player_id;
    }
}

fn remove_kicked_players(gamestate: &mut GameState, json_players: &[JSONPlayer]) {
    let players_to_kick = gamestate.players.iter()
        .filter(|(_, player)| !json_players.iter().any(|json| json.color == player.color))
        .map(|(id, _)| *id)
        .collect::<Vec<_>>();

    for player in players_to_kick {
        gamestate.remove_player(player);
    }
}

fn set_player_scores(gamestate: &mut GameState, json_players: &[JSONPlayer]) {
    for json_player in json_players {
        let player = gamestate.get_player_by_color_mut(json_player.color).unwrap();
        player.score = json_player.score;
    }
}

fn place_penguins(gamestate: &mut GameState, json_players: &[JSONPlayer]) {
    for json_player in json_players {
        let places = util::map_slice(&json_player.places,
            |place| gamestate.board.get_tile_id(place[1], place[0]).unwrap());

        // The players list in the json may be in a different order than the
        // immutable turn_order in teh gamestate, so we have to manually search
        // for the corresponding player of the same color on each iteration.
        let player = gamestate.get_player_by_color_mut(json_player.color).unwrap();

        for i in 0 .. player.penguins.len() {
            player.penguins[i].tile_id = places.get(i).copied();
        }
    }
}
