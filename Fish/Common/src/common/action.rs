//! The code in this file represents the actions a player can make
//! when it is their turn. Currently, players can only moves, though
//! they will also eventually be able to place penguins.
use crate::common::penguin::PenguinId;
use crate::common::tile::TileId;
use crate::common::boardposn::BoardPosn;
use crate::common::player::PlayerColor;
use crate::common::gamestate::GameState;

use serde::{ Serialize, Deserialize };

/// A Move is the main action a player can take on their turn.
/// It consists of a penguin to move and a tile to move it to.
/// 
/// For a move to be valid, there are several conditions that must
/// be met: the penguin must belong to the player, the tile it is moving
/// on must be reachable from the penguin's current tile, etc. See
/// GameState::move_avatar_for_player for more details on making moves
/// and the conditions for which they are valid.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Move {
    pub penguin_id: PenguinId,
    pub tile_id: TileId,
}

unsafe impl Send for Move {}
unsafe impl Sync for Move {}

impl Move {
    pub fn new(penguin_id: PenguinId, tile_id: TileId) -> Move {
        Move { penguin_id, tile_id }
    }
}

/// Represents a move that has been  made by a given player
#[derive(Copy, Clone)]
pub struct PlayerMove {
    pub mover: PlayerColor,
    pub from: BoardPosn,
    pub to: BoardPosn,
}

impl PlayerMove {
    pub fn new(mover: PlayerColor, move_: Move, state: &GameState) -> Option<PlayerMove> {
        let from = state.get_penguin_tile_position(move_.penguin_id)?;
        let to = state.board.get_tile_position(move_.tile_id);
        Some(PlayerMove { mover, from, to })
    }
}

/// A Placement is the TileId to place a penguin onto.
/// This struct represents the data needed to send a PlacePenguin
/// message to the server.
/// 
/// The player does not get to choose the penguin that is placed,
/// when sending a PlacePenguin message to the server the server
/// will validate the message came from the player whose turn it
/// currently is then make the move if possible. See
/// GameState::place_avatar_for_player for more info on invalid placements.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Placement {
    pub tile_id: TileId,
}

unsafe impl Send for Placement {}
unsafe impl Sync for Placement {}

impl Placement {
    pub fn new(tile_id: TileId) -> Placement {
        Placement { tile_id }
    }
}

/// Represents any action a player may take on their turn.
/// 
/// Used for serializing and deserializing player actions for
/// communicating with the server.
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Action {
    PlacePenguin(Placement),
    MovePenguin(Move)
}

unsafe impl Send for Action {}
unsafe impl Sync for Action {}

impl Action {
    pub fn as_placement(self) -> Option<Placement> {
        match self {
            Action::PlacePenguin(placement) => Some(placement),
            _ => None,
        }
    }

    pub fn as_move(self) -> Option<Move> {
        match self {
            Action::MovePenguin(move_) => Some(move_),
            _ => None,
        }
    }
}
