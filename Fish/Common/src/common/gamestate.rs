//! The gamestate module defines the SharedGameState that will be
//! serialized by the server and sent to each client - informing
//! them of any updates to the game. The GameState itself is a
//! shared mutable pointer which in the client is shared between
//! the communication layer (TBD) and the ui layer.
use crate::common::board::Board;
use crate::common::tile::{ TileId, Tile };

use std::rc::Rc;
use std::cell::RefCell;

// Rc<RefCell<T>> gives a copiable, mutable reference to its T
pub type GameState = Rc<RefCell<SharedGameState>>;

/// The SharedGameState contains the entirety of the current state
/// of the game. It is meant to be serialized into json from the server
/// and sent to each client to deserialize to receive the updated game
/// state each turn. The SharedGameState is rendering-agnostic, so each
/// client is free to render the SharedGameState however it wishes.
pub struct SharedGameState {
    pub board: Board,
}

/// Convenience function for creating a new gamestate containing a
/// board with the given specifications.
pub fn new_gamestate(rows: u32, columns: u32, fish_per_tile: u8) -> GameState {
    let board = Board::with_no_holes(rows, columns, fish_per_tile);
    let shared_state = SharedGameState { board };
    Rc::new(RefCell::new(shared_state))
}

impl SharedGameState {
    /// Retrieve a tile by it's ID. Will return None if the id
    /// does not reference any existing tile. This can happen
    /// if the tile was removed and has become a hole in the board.
    pub fn get_tile(&self, tile_id: TileId) -> Option<&Tile> {
        self.board.tiles.get(&tile_id)
    }
}
