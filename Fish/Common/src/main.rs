mod client;
mod common;

use common::gamestate::GameState;
use common::board::Board;
use common::tile::TileId;

fn main() {
    let board = Board::with_holes(6, 4, vec![(2, 3).into()], 3);
    let gamestate = GameState::new(1, board, 4);
    {
        let mut gamestate = gamestate.borrow_mut();
        let (&player_id, player) = gamestate.players.iter().nth(1).unwrap();
        let penguin_id = player.penguins[0].penguin_id;
        gamestate.place_avatar_for_player(player_id, penguin_id, TileId(0));
    }
    client::show_ui(gamestate);
}
