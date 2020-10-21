use crate::common::penguin::PenguinId;
use crate::common::tile::TileId;

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Move {
    pub penguin_id: PenguinId,
    pub tile_id: TileId,
}

impl Move {
    pub fn new(penguin_id: PenguinId, tile_id: TileId) -> Move {
        Move { penguin_id, tile_id }
    }
}