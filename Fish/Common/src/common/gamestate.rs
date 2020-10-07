use crate::common::board::Board;
use crate::common::tile::{ TileId, Tile };

use std::rc::Rc;
use std::cell::RefCell;

pub type GameState = Rc<RefCell<SharedGameState>>;

pub struct SharedGameState {
    pub board: Board,
}

pub fn new_gamestate(rows: u32, columns: u32, fish_per_tile: u8) -> GameState {
    let board = Board::with_no_holes(rows, columns, fish_per_tile);
    let shared_state = SharedGameState { board };
    Rc::new(RefCell::new(shared_state))
}

impl SharedGameState {
    pub fn get_tile(&self, tile_id: TileId) -> Option<&Tile> {
        self.board.tiles.get(&tile_id)
    }
}
