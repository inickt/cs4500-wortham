use crate::common::penguin::{ Penguin, PenguinId };
use crate::common::board::Board;
use crate::common::tile::TileId;
use crate::common::util::map_slice;

use std::sync::atomic::{ AtomicUsize, Ordering };

/// Amount of players generated in the current instance of this program.
/// Used for setting unique PlayerIds for each player.
static TOTAL_PLAYER_COUNT: AtomicUsize = AtomicUsize::new(0);

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct PlayerId(pub usize);

pub struct Player {
    pub player_id: PlayerId,
    pub penguins: Vec<Penguin>,
}

impl Player {
    pub fn new(penguins: Vec<Penguin>) -> Player {
        let player_id = PlayerId(TOTAL_PLAYER_COUNT.fetch_add(1, Ordering::SeqCst));
        Player { player_id, penguins }
    }

    /// Places one of this players' penguins to a new location on the given board.
    /// Returns Some(()) if the move succeeded, None if it failed. This approach is used over
    /// booleans to reduce code nesting when dealing with Option types, using the "?" operator.
    pub fn place_penguin(&mut self, penguin_id: PenguinId, tile_id: TileId, board: &Board) -> Option<()> {
        let penguin = self.find_penguin_mut(penguin_id)?;

        // Penguin must not yet be on a tile to be initially placed.
        // move_penguin should be used to move an already-placed penguin.
        if penguin.tile != None {
            None
        } else {
            let to_tile = board.tiles.get(&tile_id)?;
            penguin.tile = Some(tile_id);
            Some(())
        }
    }

    /// Moves one of this players' penguins to a new location on the given board.
    /// Returns Some(()) if the move succeeded, None if it failed. This approach is used over
    /// booleans to reduce code nesting when dealing with Option types, using the "?" operator.
    pub fn move_penguin(&mut self, penguin_id: PenguinId, to_tile_id: TileId, board: &Board) -> Option<()> {
        let penguin = self.find_penguin_mut(penguin_id)?;
        let from_tile = board.tiles.get(&penguin.tile?)?;
        let to_tile = board.tiles.get(&to_tile_id)?;

        if from_tile.can_reach(board, to_tile) {
            penguin.tile = Some(to_tile_id);
            Some(())
        } else {
            None
        }
    }

    fn find_penguin_mut(&mut self, penguin_id: PenguinId) -> Option<&mut Penguin> {
        self.penguins.iter_mut().find(|p| penguin_id == p.penguin_id)
    }
}

#[test]
fn test_place_penguin() {
    // 0   3   6
    //   1   4   7
    // 2   5   8
    let mut board = Board::with_no_holes(3, 3, 3);
    board.remove_tile(TileId(5));

    let penguins = vec![Penguin::new(), Penguin::new()];
    let penguin_ids = map_slice(&penguins, |penguin| penguin.penguin_id);
    let mut player = Player::new(penguins);

    let unowned_penguin = Penguin::new();

    // Player tried to place down a penguin they don't own
    assert_eq!(player.place_penguin(unowned_penguin.penguin_id, TileId(4), &board), None);

    // Player places a penguin at a valid spot
    assert_eq!(player.place_penguin(penguin_ids[0], TileId(4), &board), Some(()));

    // Placing an already-placed penguin is invalid
    assert_eq!(player.place_penguin(penguin_ids[0], TileId(4), &board), None);

    // Player tried to place a penguin at an invalid location
    assert_eq!(player.place_penguin(penguin_ids[1], TileId(10), &board), None);

    // Player tried to place a penguin at a hole
    assert_eq!(player.place_penguin(penguin_ids[1], TileId(5), &board), None);
}

#[test]
fn test_move_penguin() {
    // 0   3   6
    //   1   4   7
    // 2   5   8
    let board = Board::with_no_holes(3, 3, 3);

    let penguins = vec![Penguin::new(), Penguin::new()];
    let penguin_ids = map_slice(&penguins, |penguin| penguin.penguin_id);
    let mut player = Player::new(penguins);

    // Reachable tiles from 0 are [0, 1, 5, 3, 6]
    let tile_0 = TileId(0);
    let reachable_tile = TileId(5);
    let unreachable_tile = TileId(2);

    // Move failed: penguin not yet placed
    assert_eq!(player.move_penguin(penguin_ids[0], tile_0, &board), None);

    player.place_penguin(penguin_ids[0], tile_0, &board);

    // Move failed: tile not reachable from tile 0
    assert_eq!(player.move_penguin(penguin_ids[0], unreachable_tile, &board), None);

    // success, penguin should now be on tile 5
    assert_eq!(player.move_penguin(penguin_ids[0], reachable_tile, &board), Some(()));

    // Finally, assert that the position of the penguin actually changed
    let penguin_pos = player.find_penguin_mut(penguin_ids[0]).and_then(|penguin| penguin.tile);
    assert_eq!(penguin_pos, Some(reachable_tile));
}