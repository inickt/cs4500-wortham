//! The board module contains the data model for the Fish game board.
//! The board knows its own width and height, and contains
//! a vector of the tiles on itself. 
//! 
//! When a board is created,
//! it assigns IDs to its tiles, each of which are computed based
//! on the x/y position of that tile. It can also compute these
//! x/y positions later on from the ID of a tile, and vice versa.
//! This process and the math used are further documented starting
//! on line 22.
use crate::common::tile::{ Tile, TileId };
use crate::common::boardposn::BoardPosn;
use std::collections::BTreeMap;

use serde::{ Serialize, Deserialize };

/// The fish game board is a mapping of tiles from their unique tile ids to
/// their Tile representation. This is essentially a Graph of tiles where each
/// tile is a node on the graph containing its adjacency list within. The Board
/// itself thus doesn't directly care about the location/ordering of any tile,
/// though it does provide convenience methods for the client graphics program
/// to get the position of a given tile for rendering. See the ASCII art in the
/// documentation for Board::with_no_holes for more information on how tile ids
/// are made and translated to/from board positions. It should be noted though
/// that a tile's adjacency list is the preferred method for working on it within
/// game rules validation code.
/// 
/// A hole is represented by the absense of that tile id in the tiles hash map.
/// Also within each tile, if their neighbor is a hole, that link will be None
/// rather than Some(TileId). Therefore to create a hole from an existing tile,
/// remove that tile from the map and unlink it from its neighbor Tiles.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Board {
    pub tiles: BTreeMap<TileId, Tile>,
    pub width: u32,
    pub height: u32,
}

impl Board {
    /// Creates a board that has the same number of fish on every tile and has no holes
    ///
    /// a 3 x 4 matrix of tiles:
    /// [0,0]     [1,0]     [2,0]              is-odd-row = 0, is-even-row = 1
    ///     [0,1]     [1,1]     [2,1]          is-odd-row = 1, is-even-row = 0
    /// [0,2]     [1,2]     [2,2]              is-odd-row = 0, is-even-row = 1
    ///     [0,3]     [1,3]    [2,3]           is-odd-row = 1, is-even-row = 0
    ///
    /// Will be assigned the following TileIds:
    /// 0   4   8
    ///   1   5   9
    /// 2   6   10
    ///   3   7   11
    ///
    /// Using these formulas to calculate the neighbors of a given tile, provided
    /// it is within bounds of the board itself:
    ///
    /// northeast tile = [x + is-odd-row, y - 1]
    /// northwest tile = [x - is-even-row, y - 1]
    /// north tile = [x, y - 2]
    /// south tile = [x, y + 2]
    /// southeast tile = [x + is-odd-row, y + 1]
    /// southwest tile = [x - is-even-row, y + 1]
    pub fn with_no_holes(rows: u32, columns: u32, fish_per_tile: usize) -> Board {
        let mut tiles = BTreeMap::new();

        // Convert row-major form to the column-major form used internally.
        // Also convert to signed representation for bounds checking later which may use negatives.
        let (width, height) = (columns as i64, rows as i64);

        for x in 0 .. width {
            for y in 0 .. height { // ids are generated in row-major order
                let is_odd_row = y % 2; // 1 if odd, 0 if not
                let is_even_row = (y + 1) % 2;
                let tile_id = Board::compute_tile_id(width, height, x, y).unwrap();

                tiles.insert(tile_id, Tile {
                    tile_id,
                    fish_count: fish_per_tile,
                    northeast: Board::compute_tile_id(width, height, x + is_odd_row, y - 1),
                    northwest: Board::compute_tile_id(width, height, x - is_even_row, y - 1),
                    north:     Board::compute_tile_id(width, height, x, y - 2),
                    south:     Board::compute_tile_id(width, height, x, y + 2),
                    southeast: Board::compute_tile_id(width, height, x + is_odd_row, y + 1),
                    southwest: Board::compute_tile_id(width, height, x - is_even_row, y + 1),
                });
            }
        }
        
        Board { tiles, width: columns, height: rows }
    }

    /// Creates a board that has holes in specific places and is set
    /// up with a minimum number of 1-fish tiles
    pub fn with_holes(rows: u32, columns: u32, mut holes: Vec<BoardPosn>, min_tiles_with_1_fish: u32) -> Board {
        let mut board = Board::with_no_holes(rows, columns, 1);

        holes.sort(); // sort in some arbitrary way to collect duplicates together
        holes.dedup(); // remove all consecutive duplicates
        let num_tiles_without_holes = rows * columns - holes.len() as u32;

        assert!(num_tiles_without_holes >= min_tiles_with_1_fish,
            "Board::with_holes was required to create a board with a minimum of {} 1 fish tiles,
             but was unable to because the maximum number of non-hole tiles it could create is only {}",
            min_tiles_with_1_fish, num_tiles_without_holes);

        for hole in holes {
            if let Some(id) = Board::compute_tile_id(columns as i64, rows as i64, hole.x as i64, hole.y as i64) {
                board.remove_tile(id);
            }
        }

        board
    }

    /// Create a Board from a 2D Vec in row-major order
    /// (a list of rows), where each entry in the matrix
    /// is a number corresponding to the number of fish on the
    /// tile at that position. If the number is 0, there
    /// is a hole at that position.
    pub fn from_tiles(tiles: Vec<Vec<u32>>) -> Board {
        let rows = tiles.len() as u32;
        let columns = tiles.get(0).map_or(0, |row| row.len()) as u32;
        
        let mut board = Board::with_no_holes(rows, columns, 1);

        for (y, row) in tiles.into_iter().enumerate() {
            for (x, fish_count) in row.into_iter().enumerate() {
                let tile = board.get_tile_mut(x as u32, y as u32).unwrap();

                if fish_count == 0 {
                    let tile_id = tile.tile_id;
                    board.remove_tile(tile_id);
                } else {
                    tile.fish_count = fish_count as usize;
                }
            }
        }

        board
    }

    /// Computes the TileId for a tile at (tile_x, tile_y) iff the tile is within the given boundaries.
    /// tile_x and tile_y are given as (col, row) rather than position in px
    fn compute_tile_id(board_width: i64, board_height: i64, tile_x: i64, tile_y: i64) -> Option<TileId> {
        if tile_x < 0 || tile_y < 0 || tile_x >= board_width || tile_y >= board_height {
            None
        } else {
            let id = tile_x * board_height + tile_y;
            Some(TileId(id as usize))
        }
    }

    /// Computes the position of a tile on this board from its id
    /// Position returned is (col, row) rather than position in px
    /// Assumes tile_id is valid for this board
    pub fn get_tile_position(&self, tile_id: TileId) -> BoardPosn {
        let x = tile_id.0 as u32 / self.height;
        let y = tile_id.0 as u32 % self.height;
        BoardPosn { x, y }
    }

    /// Returns the TileId of the tile at tile_x, tile_y on this board
    /// or None if it's a hole or out of bounds
    pub fn get_tile_id(&self, tile_x: u32, tile_y: u32) -> Option<TileId> {
        self.get_tile(tile_x, tile_y).map(|tile| tile.tile_id)
    }

    /// Returns the tile at tile_x, tile_y on this board
    /// Returns None if hole or out of bounds
    pub fn get_tile(&self, tile_x: u32, tile_y: u32) -> Option<&Tile> {
        let expected_tile_id = Board::compute_tile_id(self.width as i64,
            self.height as i64, tile_x as i64, tile_y as i64)?;

        self.tiles.get(&expected_tile_id)
    }

    /// Returns a mutable reference to the tile at tile_x, tile_y on this board
    /// Returns None if hole or out of bounds
    pub fn get_tile_mut(&mut self, tile_x: u32, tile_y: u32) -> Option<&mut Tile> {
        let expected_tile_id = Board::compute_tile_id(self.width as i64,
            self.height as i64, tile_x as i64, tile_y as i64)?;

        self.tiles.get_mut(&expected_tile_id)
    }

    /// Removes a given Tile from the board if possible.
    /// Returns the amount of fish on the tile, or 0 if a tile was not removed.
    pub fn remove_tile(&mut self, tile_id: TileId) -> usize {
        if let Some(tile) = self.tiles.remove(&tile_id) {
            let fish_count = tile.get_fish_count();
            tile.unlink_from_neighbors(self);
            fish_count
        } else {
            0
        }
    }
}

// Can we use Board::with_no_holes to initialize tiles?
// Do these tiles get arranged in the right order and
// with the right amount of fish?
#[test]
fn test_board_with_no_holes() {
    let b = Board::with_no_holes(3, 2, 4);
    // IDs arrangement
    // 0   3
    //   1   4
    // 2   5
    assert_eq!(b.tiles.len(), 6);
    assert_eq!(b.width, 2);
    assert_eq!(b.height, 3);
    assert_eq!(b.tiles[&TileId(0)].southeast, Some(TileId(1)));
    assert_eq!(b.tiles[&TileId(3)].southwest, Some(TileId(1)));
    assert_eq!(b.tiles[&TileId(2)].northeast, Some(TileId(1)));
    assert_eq!(b.tiles[&TileId(1)].northwest, Some(TileId(0)));
    assert_eq!(b.tiles[&TileId(5)].north, Some(TileId(3)));
    assert_eq!(b.tiles[&TileId(0)].south, Some(TileId(2)));

    for i in 0 .. 6 {
        assert_eq!(b.tiles[&TileId(i)].fish_count, 4);
    }
}

// Can we use Board::with_holes to initialize tiles?
// Do these tiles get arranged in the right order and
// with the right amount of fish? Are the holes present?
#[test]
fn test_board_with_holes() {
    let holes = vec![(1, 0).into(), (1, 2).into()];
    let b = Board::with_holes(3, 2, holes, 4);
    // IDs arrangement
    // 0   -
    //   1   4
    // 2   -
    assert_eq!(b.tiles.len(), 4);
    assert_eq!(b.width, 2);
    assert_eq!(b.height, 3);
    assert_eq!(b.tiles[&TileId(0)].southeast, Some(TileId(1)));
    assert_eq!(b.tiles[&TileId(1)].northwest, Some(TileId(0)));
    assert_eq!(b.tiles[&TileId(2)].southwest, None); // out of bounds
    assert_eq!(b.tiles[&TileId(1)].northeast, None); // hole
    assert_eq!(b.tiles[&TileId(2)].northeast, Some(TileId(1)));
    assert_eq!(b.tiles[&TileId(2)].southeast, None); // out of bounds
    assert_eq!(b.tiles[&TileId(4)].southeast, None); // out of bounds
    assert_eq!(b.tiles[&TileId(4)].southwest, None); // hole
    assert_eq!(b.tiles[&TileId(2)].north, Some(TileId(0)));
    assert_eq!(b.tiles[&TileId(0)].south, Some(TileId(2)));
    assert_eq!(b.tiles.get(&TileId(3)), None); // hole
    assert_eq!(b.tiles.get(&TileId(5)), None); // hole

    // has 4 tiles with 1 fish on them
    assert_eq!(b.tiles[&TileId(0)].fish_count, 1);
    assert_eq!(b.tiles[&TileId(1)].fish_count, 1);
    assert_eq!(b.tiles[&TileId(2)].fish_count, 1);
    assert_eq!(b.tiles[&TileId(4)].fish_count, 1);
}

// Can we correctly compute a TileId from a board position?
#[test]
fn test_board_get_tile_id() {
    let (height, width) = (3, 4);
    // 3 x 4 board should look like:
    // 0    3    6    9
    //   1    4    7    10
    // 2    5    8    11
    let mut expected_id = 0; // counts down columns, then move over a row and repeat
    for x in 0 .. width {
        for y in 0 .. height {
            assert_eq!(
                Board::compute_tile_id(width, height, x, y), 
                Some(TileId(expected_id))
            );
            expected_id += 1;
        }
    }

    // out of bounds
    assert_eq!(Board::compute_tile_id(4, 3, -1, 1), None);
    assert_eq!(Board::compute_tile_id(4, 3, 1, -1), None);
    assert_eq!(Board::compute_tile_id(4, 3, 5, 1), None);
    assert_eq!(Board::compute_tile_id(4, 3, 1, 8), None);
}

// Can we correctly compute the position of a tile from its TileId?
#[test]
fn test_board_get_tile_position() {
    // This 2x3 board should look like:
    // 0    2    4
    //    1    3    5
    let b = Board::with_no_holes(2, 3, 3);
    assert_eq!(b.get_tile_position(TileId(0)), (0,0).into());
    assert_eq!(b.get_tile_position(TileId(1)), (0,1).into());
    assert_eq!(b.get_tile_position(TileId(2)), (1,0).into());
    assert_eq!(b.get_tile_position(TileId(3)), (1,1).into());
    assert_eq!(b.get_tile_position(TileId(4)), (2,0).into());
    assert_eq!(b.get_tile_position(TileId(5)), (2,1).into());
}

// Can we remove a tile from a board?
#[test]
fn test_board_remove_tile() {
    // This 2x3 board should look like:
    // 0    2    4
    //    1    3    5
    let mut b = Board::with_no_holes(2, 3, 3);
    let tile_to_remove = &b.tiles[&TileId(2)];
    let tile_neighbor_se = &b.tiles[&TileId(3)];
    let tile_neighbor_sw = &b.tiles[&TileId(1)];
    let old_num_tiles = b.tiles.len();
    assert_eq!(Some(tile_neighbor_se.tile_id), tile_to_remove.southeast);
    assert_eq!(Some(tile_neighbor_sw.tile_id), tile_to_remove.southwest);
    assert_eq!(tile_neighbor_se.northwest, Some(tile_to_remove.tile_id));
    assert_eq!(tile_neighbor_sw.northeast, Some(tile_to_remove.tile_id));
    // all tiles have 3 fish on them, and remove_tile returns # fish on tile
    assert_eq!(b.remove_tile(TileId(2)), 3);
    let tile_neighbor_se = &b.tiles[&TileId(3)];
    let tile_neighbor_sw = &b.tiles[&TileId(1)];
    assert_eq!(b.tiles.len(), old_num_tiles - 1);
    assert_eq!(b.tiles.get(&TileId(2)), None);
    assert_eq!(tile_neighbor_se.northwest, None);
    assert_eq!(tile_neighbor_sw.northeast, None);
}
