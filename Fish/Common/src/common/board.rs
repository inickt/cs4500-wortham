use crate::common::tile::{ Tile, TileId };
use crate::common::boardposn::BoardPosn;
use std::collections::HashMap;

pub struct Board {
    pub tiles: HashMap<TileId, Tile>,
    width: u32,
    height: u32,
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
    pub fn with_no_holes(rows: u32, columns: u32, fish_per_tile: u8) -> Board {
        let mut tiles = HashMap::new();

        // Convert row-major form to the column-major form used internally.fish_per_tile
        // Also convert to signed representation for bounds checking later which may use negatives.
        let (width, height) = (columns as i64, rows as i64);

        for x in 0 .. width {
            for y in 0 .. height { // ids are generated in row-major order
                let is_odd_row = y % 2; // 1 if odd, 0 if not
                let is_even_row = (y + 1) % 2;
                let tile_id = Board::get_tile_id(width, height, x, y).unwrap();

                tiles.insert(tile_id, Tile {
                    tile_id,
                    fish_count: fish_per_tile,
                    northeast: Board::get_tile_id(width, height, x + is_odd_row, y - 1),
                    northwest: Board::get_tile_id(width, height, x - is_even_row, y - 1),
                    north:     Board::get_tile_id(width, height, x, y - 2),
                    south:     Board::get_tile_id(width, height, x, y + 2),
                    southeast: Board::get_tile_id(width, height, x + is_odd_row, y + 1),
                    southwest: Board::get_tile_id(width, height, x - is_even_row, y + 1),
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
            if let Some(id) = Board::get_tile_id(columns as i64, rows as i64, hole.x as i64, hole.y as i64) {
                board.remove_tile(id);
            }
        }

        board
    }

    /// Returns the TileId for the tile at (tile_x, tile_y) iff the tile is within the bounds of the board.
    /// tile_x and tile_y are given as (col, row) rather than position in px
    fn get_tile_id(board_width: i64, board_height: i64, tile_x: i64, tile_y: i64) -> Option<TileId> {
        if tile_x < 0 || tile_y < 0 || tile_x >= board_width || tile_y >= board_height {
            None
        } else {
            let id = tile_x * board_height + tile_y;
            Some(TileId(id as usize))
        }
    }

    /// Computes the position of a tile on this board from its id
    /// Position returned is (col, row) rather than position in px
    pub fn get_tile_position(&self, tile_id: TileId) -> BoardPosn {
        let x = tile_id.0 as u32 / self.height;
        let y = tile_id.0 as u32 % self.height;
        BoardPosn { x, y }
    }

    /// Removes a given Tile from the board if possible.
    /// Returns true if the tile was successfully removed.
    pub fn remove_tile(&mut self, tile_id: TileId) -> bool {
        if let Some(tile) = self.tiles.remove(&tile_id) {
            tile.unlink_from_neighbors(self);
            true
        } else {
            false
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
    assert_eq!(b.tiles.get(&TileId(3)), None);
    assert_eq!(b.tiles.get(&TileId(5)), None);

    assert_eq!(b.tiles[&TileId(0)].fish_count, 1);
    assert_eq!(b.tiles[&TileId(1)].fish_count, 1);
    assert_eq!(b.tiles[&TileId(2)].fish_count, 1);
    assert_eq!(b.tiles[&TileId(4)].fish_count, 1);
}