use crate::common::board::Board;
use crate::common::tile::{ TileId, Tile };
use crate::common::boardposn::BoardPosn;

use std::rc::Rc;
use std::cell::RefCell;

// Rc<RefCell<T>> gives us a copiable mutable reference to its T
pub type GameState = Rc<RefCell<SharedGameState>>;

pub struct SharedGameState {
    pub board: Board,
}

pub fn new_gamestate(rows: u32, columns: u32, fish_per_tile: u8) -> GameState {
    let board = Board::with_no_holes(rows, columns, 5);
    let shared_state = SharedGameState { board };
    Rc::new(RefCell::new(shared_state))
}

impl SharedGameState {
    pub fn get_tile(&self, tile_id: TileId) -> Option<&Tile> {
        self.board.tiles.get(&tile_id)
    }
}
