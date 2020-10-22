mod client;
mod common;

use common::gamestate::GameState;
use common::board::Board;
use common::tile::TileId;

fn main() {
    let board = Board::with_no_holes(3, 3, 3);
    let state = GameState::new(2, board, 2);

    {
        let mut state = state.borrow_mut();

        let mut tile_ids: Vec<_> = state.board.tiles.iter().map(|(tile_id, _)| *tile_id).collect();
        tile_ids.sort();
        tile_ids.reverse();

        for (player_id, penguin_id) in state.all_penguins() {
            let tile_id = tile_ids.pop().unwrap();
            state.place_avatar_without_changing_turn(player_id, penguin_id, tile_id);
        }
    }

    client::show_ui(state);
}
