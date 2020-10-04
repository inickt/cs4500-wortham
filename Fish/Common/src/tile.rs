use std::hash::{ Hash, Hasher };
use crate::direction::Direction;
use crate::board::Board;

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct TileId(pub usize);

/// Represents a single tile on the game board.
#[derive(Eq)]
pub struct Tile {
    /// A Tile's tile_id is it's unique index in the Board.tiles Vec
    pub tile_id: TileId,

    /// How many fish are currently on this tile
    pub fish_count: u8,

    // Adjacent tiles in each direction. If there is no tile
    // or a hole in the given direction then the tile is None
    pub northeast: Option<TileId>,
    pub northwest: Option<TileId>,
    pub east: Option<TileId>,
    pub west: Option<TileId>,
    pub southeast: Option<TileId>,
    pub southwest: Option<TileId>,
}

impl PartialEq for Tile {
    fn eq(&self, other: &Tile) -> bool {
        self.tile_id == other.tile_id
    }
}

impl Hash for Tile {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.tile_id.hash(state);
    }
}

impl Tile {
    pub fn new(tile_id: usize, fish_count: u8) -> Tile {
        Tile {
            tile_id: TileId(tile_id),
            fish_count,
            northeast: None,
            northwest: None,
            east: None,
            west: None,
            southeast: None,
            southwest: None,
        }
    }

    /// Given a Tile and a Direction, return the first tile in the given
    /// direction from the starting tile, or None if there is no tile there.
    fn get_neighbor<'b>(&self, board: &'b Board, direction: Direction) -> Option<&'b Tile> {
        self.get_neighbor_id(direction)
            .and_then(|id| board.tiles.get(id))
    }

    /// Mutable version of the above function. Returns Some(mut neighbor) if there is a
    /// neighbor in the given direction, otherwise this returns None.
    fn get_neighbor_mut<'b>(&self, board: &'b mut Board, direction: Direction) -> Option<&'b mut Tile> {
        self.get_neighbor_id(direction)
            .and_then(move |id| board.tiles.get_mut(&id))
    }

    /// Given a Tile and a Direction, get the ID of the neighbor of the tile in that direction.
    fn get_neighbor_id<'b>(&self, direction: Direction) -> Option<&TileId> {
        match direction {
            Direction::Northeast => self.northeast.as_ref(),
            Direction::Northwest => self.northwest.as_ref(),
            Direction::East => self.east.as_ref(),
            Direction::West => self.west.as_ref(),
            Direction::Southeast => self.southeast.as_ref(),
            Direction::Southwest => self.southwest.as_ref(),
        }
    }

    /// Mutable version of the above function. This returns a mutable reference to the Option itself
    /// rather than an Option<&mut TileId> so that the neighbor can be set to None when removing a tile.
    fn get_neighbor_id_mut(&mut self, direction: Direction) -> &mut Option<TileId> {
        match direction {
            Direction::Northeast => &mut self.northeast,
            Direction::Northwest => &mut self.northwest,
            Direction::East => &mut self.east,
            Direction::West => &mut self.west,
            Direction::Southeast => &mut self.southeast,
            Direction::Southwest => &mut self.southwest,
        }
    } 

    /// Return a Vec of all tiles that a reachable via a straight line from the
    /// given tile. The given tile is considered reachable from itself as well.
    pub fn all_reachable_tiles<'b>(&'b self, board: &'b Board) -> Vec<&'b Tile> {
        Direction::iter()
            // filter out directions without neighbors. For directions with neighbors,
            // return all neighbors in that direction.
            .filter_map(|direction| {
                match self.get_neighbor(board, direction) {
                    Some(neighbor) => Some(neighbor.all_reachable_tiles_in_direction(board, direction)),
                    None => None
                }
            })
            // Then collect all the tiles in each direction into a single Vec
            .fold(vec![self], |mut all_tiles, mut tiles_in_direction| {
                all_tiles.append(&mut tiles_in_direction);
                all_tiles
            })
    }

    /// Helper function for all_reachable_tiles.
    /// Returns a Vec of all tiles reachable from a given direction, including self.
    fn all_reachable_tiles_in_direction<'b>(&'b self, board: &'b Board, direction: Direction) -> Vec<&'b Tile> {
        if let Some(tile) = self.get_neighbor(board, direction) {
            let mut reachable_tiles = tile.all_reachable_tiles_in_direction(board, direction);
            reachable_tiles.push(self);
            reachable_tiles
        } else {
            vec![self]
        }
    }

    /// Returns true if endpoint can be reached in a straight line from this Tile.
    pub fn can_reach(&self, board: &Board, endpoint: &Tile) -> bool {
        self.all_reachable_tiles(board).contains(&endpoint)
    }

    /// Sets neighbors' references of this Tile to None, effectively removing it from the Tile set.
    /// Note that because this function consumes self you cannot call it without already removing
    /// the tile from the board.
    pub fn unlink_from_neighbors(mut self, board: &mut Board) {
        for direction in Direction::iter() {
            if let Some(neighbor) = self.get_neighbor_mut(board, direction) {
                *neighbor.get_neighbor_id_mut(direction.opposite()) = None;
            }
        }
    }
}
