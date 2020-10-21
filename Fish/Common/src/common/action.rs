//! The code in this file represents the actions a player can make
//! when it is their turn. Currently, players can only moves, though
//! they will also eventually be able to place penguins.
use crate::common::penguin::PenguinId;
use crate::common::tile::TileId;

/// A Move is the main action a player can take on their turn.
/// It consists of a penguin to move and a tile to move it to.
/// 
/// For a move to be valid, there are several conditions that must
/// be met: the penguin must belong to the player, the tile it is moving
/// on must be reachable from the penguin's current tile, etc. See
/// GameState::move_avatar_for_player for more details on making moves
/// and the conditions for which they are valid.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Move {
    pub penguin_id: PenguinId,
    pub tile_id: TileId,
}

impl Move {
    pub fn new(penguin_id: PenguinId, tile_id: TileId) -> Move {
        Move { penguin_id, tile_id }
    }
}