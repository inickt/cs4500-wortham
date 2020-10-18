/// The code in this file implements the Penguin in-game
/// avatars' data representation and business logic.
use crate::common::board::Board;
use crate::common::tile::TileId;
use std::collections::HashSet;
use std::sync::atomic::{ AtomicUsize, Ordering };

/// Amount of penguins generated in the current instance of this program.
/// Used for setting unique PenguinIds for each penguin.
static TOTAL_PENGUIN_COUNT: AtomicUsize = AtomicUsize::new(0);

/// Id for a Penguin. First penguin uid is 1.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct PenguinId(usize);

/// Represents a single Penguin in the Fish game, including its position
/// on the board and a unique ID. Its position can be None, meaning
/// it is not placed yet, or Some(BoardPosn), meaning it's placed at
/// the BoardPosn on the game board.
#[derive(Clone, Debug)]
pub struct Penguin {
    pub penguin_id: PenguinId,
    /// INVARIANT: tile_id will always be a valid tile in this Tile's Board
    /// i.e. never a hole or out of bounds
    pub tile_id: Option<TileId>,
}

impl Penguin {
    /// Creates a new penguin with a unique PenguinId, starting at 1.
    /// The penguin is initially unplaced, represented with None
    /// as its BoardPosn.
    pub fn new() -> Penguin {
        let id = TOTAL_PENGUIN_COUNT.fetch_add(1, Ordering::SeqCst);

        Penguin {
            penguin_id: PenguinId(id),
            tile_id: None,
        }
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
    pub fn can_place(&self) -> bool {
        self.tile_id.is_none()
    }
}

#[test]
fn test_new() {
    let penguins = vec![Penguin::new(), Penguin::new(), Penguin::new(), Penguin::new()];

    for (i, penguin) in penguins.iter().enumerate() {
        assert_eq!(penguin.tile_id, None);
        for other_penguin in penguins[i+1..].iter() {
            assert_ne!(penguin.penguin_id, other_penguin.penguin_id);
        }
    }
}