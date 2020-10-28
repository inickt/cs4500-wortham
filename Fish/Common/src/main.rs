mod client;
mod common;

use common::gamestate::GameState;
use common::board::Board;
use client::strategy::{move_penguin_minmax, place_penguin_zigzag};
use std::rc::Rc;
use std::cell::RefCell;

fn main() {
    let board = Board::with_no_holes(3, 5, 3);
    let mut state = GameState::new(2, board, 2);

    let mut tile_ids: Vec<_> = state.board.tiles.iter().map(|(tile_id, _)| *tile_id).collect();
    tile_ids.sort();
    tile_ids.reverse();

    for (player_id, penguin_id) in state.all_penguins() {
        place_penguin_zigzag(&mut state);
    }

    move_penguin_minmax(&mut state, 1);

    let state = Rc::new(RefCell::new(state));
    client::show_ui(state);
}
