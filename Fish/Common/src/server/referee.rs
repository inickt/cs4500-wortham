//! This file contains all logic and data regarding the Referee component,
//! which runs complete games of Fish.
use crate::common::board::Board;
use crate::common::gamestate::GameState;
use crate::common::game_tree::GameTree;
use crate::common::player::PlayerId;
use serde::Serialize;

use std::io::{ Read, Write };
use std::collections::HashMap;

pub struct ServerPlayer {
    input_stream: Box<dyn Read>,
    output_stream: Box<dyn Write>,
}

/// ????
struct Referee {
    players: HashMap<PlayerId, ServerPlayer>,
    state: GameState,
}

/// Runs a complete game of Fish, setting up the board and
/// waiting for player input for gameplay to occur, and terminating
/// when a player (or multiple) have won. Check out Planning/player-protocol.md
/// for more information on the Fish game.
/// 
/// Returns the winning players of the game
pub fn run_game(players: Vec<ServerPlayer>, board: Option<Board>) -> Vec<PlayerId> {
    let board = board.unwrap_or(Board::with_no_holes(5, 5, 3));
    let mut referee = Referee::new(players, board);

    while !referee.is_game_over() {
        referee.send_gamestate_to_all_players();
        referee.do_player_turn();
    }

    referee.state.winning_players
}

impl Referee {
    fn new(players: Vec<ServerPlayer>, board: Board) -> Referee {
        let state = GameState::new(board, players.len());

        Referee {
            players: state.turn_order.iter().copied().zip(players.into_iter()).collect(),
            state,
        }
    }
    
    fn send_gamestate_to_all_players(&mut self) {
        for player in self.players.values_mut() {
            let serialized = serde_json::to_string(&self.state).unwrap();
            player.output_stream.write(serialized.as_bytes());
        }
    }

    fn do_player_turn(&mut self) {
        let current_player = self.players.get_mut(&self.state.current_turn).unwrap();
        unimplemented!("Need to read from current_player.input_stream");
    }

    fn is_game_over(&self) -> bool {
        self.state.is_game_over()
    }
}