//! The tile module represents the data model and some business logic
//! for the tiles of the fish game board.
//! 
//! Tiles are represented as a graph, with each knowing its
//! six neighbors: north, south, northeast, southeast, northwest,
//! and southwest. The tiles also have unique IDs and counts
//! of the amount of fish on them.

use std::collections::HashSet;
use std::hash::{ Hash, Hasher };
use std::fmt::Debug;
use crate::common::direction::Direction;
use crate::common::board::Board;

use serde::{ Serialize, Deserialize };

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct TileId(pub usize);

/// Represents a single tile on the game board.
#[derive(Clone, Eq, Serialize, Deserialize)]
pub struct Tile {
    /// A Tile's tile_id is it's unique index in the Board.tiles Vec
    pub tile_id: TileId,

    /// How many fish are currently on this tile
    pub fish_count: usize,

    // Adjacent tiles in each direction. If there is no tile
    // or a hole in the given direction then the tile is None
    pub northeast: Option<TileId>,
    pub northwest: Option<TileId>,
    pub north: Option<TileId>,
    pub south: Option<TileId>,
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

impl Debug for Tile {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        write!(f, "tile_id: {}, fish: {}", self.tile_id.0, self.fish_count)
    }
}

impl Tile {
    pub fn new(tile_id: usize, fish_count: usize) -> Tile {
        Tile {
            tile_id: TileId(tile_id),
            fish_count,
            northeast: None,
            northwest: None,
            north: None,
            south: None,
            southeast: None,
            southwest: None,
        }
    }

    /// Get the amount of fish on this Tile
    pub fn get_fish_count(&self) -> usize {
        self.fish_count
    }

    /// Return the first tile in the given direction from the starting tile, 
    /// or None if there is no tile there.
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
    pub fn get_neighbor_id<'b>(&self, direction: Direction) -> Option<&TileId> {
        match direction {
            Direction::Northeast => self.northeast.as_ref(),
            Direction::Northwest => self.northwest.as_ref(),
            Direction::North => self.north.as_ref(),
            Direction::South => self.south.as_ref(),
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
            Direction::North => &mut self.north,
            Direction::South => &mut self.south,
            Direction::Southeast => &mut self.southeast,
            Direction::Southwest => &mut self.southwest,
        }
    } 

    /// Return a Vec of all tiles that a reachable via a straight line from the
    /// given tile. The starting tile is not considered reachable from itself.
    pub fn all_reachable_tiles<'b>(&'b self, board: &'b Board, occupied_tiles: &HashSet<TileId>) -> Vec<&'b Tile> {
        Direction::iter()
            // filter out directions without neighbors. For directions with neighbors,
            // return all neighbors in that direction.
            .filter_map(|direction| {
                self.get_neighbor(board, direction).map(|neighbor|
                    neighbor.all_reachable_tiles_in_direction(board, direction, occupied_tiles))
            })
            // Then collect all the tiles in each direction into a single Vec
            .fold(vec![], |mut all_tiles, mut tiles_in_direction| {
                all_tiles.append(&mut tiles_in_direction);
                all_tiles
            })
    }

    /// Helper function for all_reachable_tiles.
    /// Returns a Vec of all tiles reachable from a given direction, including self.
    /// Returns empty vec if self is occupied.
    pub fn all_reachable_tiles_in_direction<'b>(&'b self, board: &'b Board, direction: Direction, occupied_tiles: &HashSet<TileId>) -> Vec<&'b Tile> {
        if occupied_tiles.contains(&self.tile_id) {
            vec![]
        } else {
            match self.get_neighbor(board, direction) {
                Some(neighbor) => {
                    let mut reachable_tiles = neighbor.all_reachable_tiles_in_direction(board, direction, occupied_tiles);
                    reachable_tiles.push(self);
                    reachable_tiles
                },
                None => vec![self],
            }
        }
    }

    /// Returns true if endpoint can be reached in a straight line from this Tile.
    pub fn can_reach(&self, board: &Board, endpoint: &Tile, occupied_tiles: &HashSet<TileId>) -> bool {
        self.all_reachable_tiles(board, occupied_tiles).contains(&endpoint)
    }

    /// Sets neighbors' references of this Tile to None, effectively removing it from the Tile set.
    /// Note that because this function consumes self you cannot call it without already removing
    /// the tile from the board.
    pub fn unlink_from_neighbors(self, board: &mut Board) {
        for direction in Direction::iter() {
            if let Some(neighbor) = self.get_neighbor_mut(board, direction) {
                *neighbor.get_neighbor_id_mut(direction.opposite()) = None;
            }
        }
    }
}

// Can we use Tile::new to initialize tiles?
#[test]
fn test_tile_new() {
    let tile = Tile::new(1, 4);
    assert_eq!(tile.southeast, None);
    assert_eq!(tile.northwest, None);
    assert_eq!(tile.fish_count, 4);
}

// Make sure tiles with same ID are equal, and with different IDs are not equal
#[test]
fn test_tile_eq() {
    let tile1 = Tile::new(1, 4);
    let tile2 = Tile::new(1, 4);
    let tile3 = Tile::new(2, 4);

    assert_eq!(tile1, tile2);
    assert_ne!(tile1, tile3);
}

// Can we get the neighbor IDs in a given direction?
#[test]
fn test_tile_get_neighbor_id() {
    let mut tile1 = Tile::new(1, 4);
    let tile2 = Tile::new(2, 4);
    tile1.southeast = Some(tile2.tile_id);
    let se = Direction::Southeast;
    assert_eq!(tile1.get_neighbor_id(se), Some(&tile2.tile_id));
}

// Can we get mutable references neighbor IDs in a given direction,
// and can we actually mutate those references?
#[test]
fn test_get_neighbor_id_mut() {
    let mut tile1 = Tile::new(1, 4);
    let tile2 = Tile::new(2, 4);
    let tile3 = Tile::new(3, 4);
    tile1.southeast = Some(tile2.tile_id);
    let se = Direction::Southeast;
    assert_eq!(tile1.get_neighbor_id_mut(se), &mut Some(tile2.tile_id));
    *tile1.get_neighbor_id_mut(se) = Some(tile3.tile_id);
    assert_ne!(tile1.get_neighbor_id_mut(se), &mut Some(tile2.tile_id));
    assert_eq!(tile1.get_neighbor_id_mut(se), &mut Some(tile3.tile_id));
}

// Can we get the neighbor of a Tile in any direction given a board?
#[test]
fn test_get_neighbor() {
    // 3 x 4 board should look like:
    // 0    3    6    9
    //   1    4    7    10
    // 2    5    8    11
    let b = Board::with_no_holes(3, 4, 4);
    let tile_5 = b.tiles.get(&TileId(5)).unwrap();
    let tile_4 = b.tiles.get(&TileId(4)).unwrap();
    let tile_3 = b.tiles.get(&TileId(3)).unwrap();
    let tile_1 = b.tiles.get(&TileId(1)).unwrap();
    assert_eq!(tile_5.get_neighbor(&b, Direction::North), Some(tile_3));
    assert_eq!(tile_3.get_neighbor(&b, Direction::South), Some(tile_5));
    assert_eq!(tile_5.get_neighbor(&b, Direction::Northeast), Some(tile_4));
    assert_eq!(tile_4.get_neighbor(&b, Direction::Southwest), Some(tile_5));
    assert_eq!(tile_5.get_neighbor(&b, Direction::Northwest), Some(tile_1));
    assert_eq!(tile_1.get_neighbor(&b, Direction::Southeast), Some(tile_5));
}

#[test]
fn test_all_reachable_tiles_in_direction() {
    let b = Board::with_no_holes(3, 4, 4);
    let tile_5 = b.tiles.get(&TileId(5)).unwrap();
    assert_eq!(tile_5.all_reachable_tiles_in_direction(&b, Direction::North, &HashSet::new()), vec![
        &b.tiles[&TileId(3)],
        &b.tiles[&TileId(5)]
    ]);
    assert_eq!(tile_5.all_reachable_tiles_in_direction(&b, Direction::Northeast, &HashSet::new()), vec![
        &b.tiles[&TileId(6)],
        &b.tiles[&TileId(4)],
        &b.tiles[&TileId(5)]
    ]);

}

#[test]
fn test_all_reachable_tiles() {
    // 3 x 4 board should look like:
    // 0    3    6    9
    //   1    4    7    10
    // 2    5    8    11
    let b = Board::with_no_holes(3, 4, 4);
    let tile_5 = b.tiles.get(&TileId(5)).unwrap();
    let expected_reachable = vec![
        &b.tiles[&TileId(6)],
        &b.tiles[&TileId(4)],
        &b.tiles[&TileId(0)],
        &b.tiles[&TileId(1)],
        &b.tiles[&TileId(3)],
    ];
    assert_eq!(tile_5.all_reachable_tiles(&b, &HashSet::new()), expected_reachable);

    let expected_reachable_with_occupied = vec![
        &b.tiles[&TileId(0)],
        &b.tiles[&TileId(1)],
        &b.tiles[&TileId(3)],
    ];
    assert_eq!(tile_5.all_reachable_tiles(&b, &vec![TileId(4)].into_iter().collect()), expected_reachable_with_occupied);
}

#[test]
fn test_all_reachable_tiles_all_directions() {
    // 5 x 3 board should look like:
    // 0    5    10
    //   1    6    11
    // 2    7    12
    //   3    8    13
    // 4    9    14
    let b = Board::with_no_holes(5, 3, 4);

    let tile_7 = b.tiles.get(&TileId(7)).unwrap();
    let expected_reachable = vec![
        &b.tiles[&TileId(10)],
        &b.tiles[&TileId(6)],
        &b.tiles[&TileId(0)],
        &b.tiles[&TileId(1)],
        &b.tiles[&TileId(5)],
        &b.tiles[&TileId(9)],
        &b.tiles[&TileId(14)],
        &b.tiles[&TileId(8)],
        &b.tiles[&TileId(4)],
        &b.tiles[&TileId(3)],
    ];
    assert_eq!(tile_7.all_reachable_tiles(&b, &HashSet::new()), expected_reachable);
}