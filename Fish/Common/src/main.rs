mod client;
mod common;

use common::gamestate::GameState;
use common::board::Board;
use common::util;
use common::boardposn::BoardPosn;
use common::tile::TileId;

fn main() {
    let board = Board::with_no_holes(2, 2, 4);
    let gamestate = GameState::new(1, board, 4);
    {
        let mut gamestate = gamestate.borrow_mut();
        let (&player_id, player) = gamestate.players.iter().nth(0).unwrap();
        let penguin_id = player.penguins[0].penguin_id;
        gamestate.place_avatar_for_player(player_id, penguin_id, TileId(0));
    }
    client::show_ui(gamestate);
}
