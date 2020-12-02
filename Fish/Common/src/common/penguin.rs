/// The code in this file implements the Penguin in-game
/// avatars' data representation and business logic.
use crate::common::board::Board;
use crate::common::tile::TileId;
use std::collections::HashSet;

use serde::{ Serialize, Deserialize };

/// Represents a single Penguin in the Fish game, including its position
/// on the board and a unique ID. Its position can be None, meaning
/// it is not placed yet, or Some(BoardPosn), meaning it's placed at
/// the BoardPosn on the game board.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Penguin {
    /// INVARIANT: tile_id will always be a valid tile in this Tile's Board
    /// i.e. never a hole or out of bounds
    pub tile_id: Option<TileId>,
}

impl Penguin {
    /// Creates a new penguin with a unique PenguinId, starting at 1.
    /// The penguin is initially unplaced, represented with None
    /// as its BoardPosn.
    pub fn new() -> Penguin {
        Penguin { tile_id: None }
    }

    /// Can this penguin move to any other tile it's not currently on?
    /// Returns false for unplaced penguins
    pub fn can_move(&self, board: &Board, occupied_tiles: &HashSet<TileId>) -> bool {
        match self.tile_id {
            Some(tile_id) => {
                // panics if the penguin's tile_id is a hole
                let tile = board.tiles.get(&tile_id).unwrap();
                tile.all_reachable_tiles(board, occupied_tiles).len() > 0
            },
            None => false,
        }
    }

    /// Can this penguin be placed on the board?
    pub fn is_placed(&self) -> bool {
        self.tile_id.is_some()
    }
}

#[test]
fn test_new() {
    assert!(Penguin::new().tile_id == None);
}
