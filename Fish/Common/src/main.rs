mod client;
mod common;

use common::gamestate::GameState;
use common::game_tree::GameTree;
use common::board::Board;
use client::strategy::{ find_minmax_move, take_zigzag_placement };
use std::rc::Rc;
use std::cell::RefCell;

fn main() {
    let board = Board::with_no_holes(3, 5, 3);
    let mut state = GameState::new(2, board, 2);

    let mut tile_ids: Vec<_> = state.board.tiles.iter().map(|(tile_id, _)| *tile_id).collect();
    tile_ids.sort();
    tile_ids.reverse();

    for (player_id, penguin_id) in state.all_penguins() {
        take_zigzag_placement(&mut state);
    }

    let move_ = find_minmax_move(&mut GameTree::new(&state), 1);
    state.move_avatar_for_current_player(move_);

    let state = Rc::new(RefCell::new(state));
    client::show_ui(state);
}
