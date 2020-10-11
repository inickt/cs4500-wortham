use crate::common::penguin::{ Penguin, PenguinId };
use crate::common::board::Board;
use crate::common::tile::TileId;

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

    fn find_penguin_mut(&self, penguin_id: PenguinId) -> Option<&mut Penguin> {
        self.penguins.iter_mut().find(|p| penguin_id == p.penguin_id)
    }
}