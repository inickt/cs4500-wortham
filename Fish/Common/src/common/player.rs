/// This file contains all the code implementing the shared
/// GameState's representation of players and their
/// game-specific information.
use crate::common::penguin::Penguin;
use crate::common::board::Board;
use crate::common::tile::TileId;
use crate::common::util;

use std::collections::HashSet;

use serde::{ Serialize, Deserialize };

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
pub struct PlayerId(pub usize);

#[allow(non_camel_case_types)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PlayerColor {
    red,
    white,
    brown,
    black,
}

impl PlayerColor {
    pub fn iter() -> impl Iterator<Item = PlayerColor> {
        vec![PlayerColor::red, PlayerColor::white, PlayerColor::brown, PlayerColor::black].into_iter()
    }
}

/// Represents an in-game player. Agnostic of the player's
/// external information, like username, connection information,
/// etc.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Player {
    pub player_id: PlayerId,
    pub penguins: Vec<Penguin>,
    pub color: PlayerColor,
    pub score: usize, // number of fish this player has collected
}

impl Player {
    /// Creates a new player with the given amount of penguins. 
    /// Initializes the player's PlayerId to be globally unique.
    pub fn new(player_id: PlayerId, color: PlayerColor, penguin_count: usize) -> Player {
        let penguins = util::make_n(penguin_count, |_| Penguin::new());
        Player { player_id, penguins, color, score: 0 }
    }

    /// Places one of this players' penguins to a new location on the given board.
    /// Returns Some(()) if the move succeeded, None if it failed. This approach is used over
    /// booleans to reduce code nesting when dealing with Option types, using the "?" operator.
    pub fn place_penguin(&mut self, tile_id: TileId, board: &Board) -> Option<()> {
        let penguin = self.find_unplaced_penguin_mut()?;

        // Penguin must not yet be on a tile to be initially placed.
        // move_penguin should be used to move an already-placed penguin.
        if penguin.tile_id != None {
            None
        } else {
            // Make sure the tile isn't a hole before setting the new tile_id
            board.tiles.get(&tile_id)?;
            penguin.tile_id = Some(tile_id);
            Some(())
        }
    }

    /// Moves the penguin at the given position to a new tile on the given board.
    /// Returns Some(()) if the move succeeded, None if it failed. This approach is used over
    /// booleans to reduce code nesting when dealing with Option types, using the "?" operator.
    pub fn move_penguin(&mut self, from_tile_id: TileId, to_tile_id: TileId, board: &Board, occupied_tiles: &HashSet<TileId>) -> Option<()> {
        let penguin = self.find_penguin_mut(from_tile_id)?;
        let from_tile = board.tiles.get(&penguin.tile_id?)?;
        let to_tile = board.tiles.get(&to_tile_id)?;

        if from_tile.can_reach(board, to_tile, occupied_tiles) {
            penguin.tile_id = Some(to_tile_id);
            Some(())
        } else {
            None
        }
    }

    /// Returns true if any of this player's penguins have any valid moves to make.
    pub fn can_move_a_penguin(&self, board: &Board, occupied_tiles: &HashSet<TileId>) -> bool {
        self.penguins.iter().any(|penguin| penguin.can_move(board, occupied_tiles))
    }

    /// Retrieves a mutable reference to a penguin by id. If the penguin does not
    /// belong to the current player this will return None.
    pub fn find_penguin_mut(&mut self, current_tile: TileId) -> Option<&mut Penguin> {
        self.penguins.iter_mut().find(|penguin| penguin.tile_id == Some(current_tile))
    }

    /// Retrieves an immutable reference to a penguin by id. If the penguin does not
    /// belong to the current player this will return None.
    pub fn find_penguin(&self, current_tile: TileId) -> Option<&Penguin> {
        self.penguins.iter().find(|penguin| penguin.tile_id == Some(current_tile))
    }

    pub fn find_unplaced_penguin_mut(&mut self) -> Option<&mut Penguin> {
        self.penguins.iter_mut().find(|penguin| penguin.tile_id == None)
    }

    pub fn has_unplaced_penguins(&self) -> bool {
        self.penguins.iter().any(|penguin| !penguin.is_placed())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_n_players(n: usize) -> Vec<Player> {
        (0..n).zip(PlayerColor::iter()).map(|(id, color)| {
            Player::new(PlayerId(id), color, 6 - n)
        }).collect()
    }

    #[test]
    fn test_new() {
        // Make 4 players with 2 penguins each
        let players = make_n_players(4);

        for (i, player) in players.iter().enumerate() {
            assert_eq!(player.penguins.len(), 2);

            for (_, other_player) in players.iter().enumerate().filter(|(j, _)| *j != i) {
                // make sure players created have unique IDs
                assert_ne!(player.player_id, other_player.player_id);
            }
        }
    }

    #[test]
    fn test_place_penguin() {
        // 0   3   6
        //   1   4   7
        // 2   5   8
        let mut board = Board::with_no_holes(3, 3, 3);
        board.remove_tile(TileId(5));

        let mut player = Player::new(PlayerId(0), PlayerColor::red, 3);

        // Player places a penguin at a valid spot
        assert_eq!(player.place_penguin(TileId(4), &board), Some(()));

        // Player tried to place a penguin at an invalid location
        assert_eq!(player.place_penguin(TileId(10), &board), None);

        // Player tried to place a penguin at a hole
        assert_eq!(player.place_penguin(TileId(5), &board), None);
    }

    #[test]
    fn test_move_penguin() {
        // 0   3   6
        //   1   4   7
        // 2   5   8
        let board = Board::with_no_holes(3, 3, 3);

        let mut player = Player::new(PlayerId(0), PlayerColor::red, 1);

        // Reachable tiles from 0 are [2, 1, 5]
        let tile_0 = TileId(0);
        let reachable_tile = TileId(5);
        let unreachable_tile = TileId(3);

        player.place_penguin(tile_0, &board);

        // Move failed: tile not reachable from tile 0
        assert_eq!(player.move_penguin(tile_0, unreachable_tile, &board, &HashSet::new()), None);

        // success, penguin should now be on tile 5
        assert_eq!(player.move_penguin(tile_0, reachable_tile, &board, &HashSet::new()), Some(()));

        // Finally, assert that the position of the penguin actually changed
        let penguin_pos = player.find_penguin_mut(reachable_tile).and_then(|penguin| penguin.tile_id);
        assert_eq!(penguin_pos, Some(reachable_tile));
    }
}
