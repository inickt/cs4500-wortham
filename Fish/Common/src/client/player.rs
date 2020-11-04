//! This file contains the implementation for an in-house AI player
//! for the Fish game.
use crate::common::action::{ Action, Move, Placement };
use crate::common::gamestate::GameState;
use crate::client::strategy::{ Strategy, ZigZagMinMaxStrategy };
use crate::common::gamephase::GamePhase;

use std::io::{ Read, Write };

use serde::Deserialize;
use serde_json::{ Deserializer, de::IoRead };

/// Represents the in-house AI player for the Fish game.
pub struct InHousePlayer {
    /// InHousePlayers always can communicate with the server through normal
    /// string (de)serialization, they don't need TcpStreams
    /// 
    /// This String gets replaced every time a new message is generated
    /// so its only content at any point is the newest client->server message
    pub output_stream: String,

    /// Contains the current phase of the game (starting, placing, moving, done),
    /// which also contains either the current GameState or GameTree depending on
    /// if we are in the Placing or Moving phase. This is the player's concept of
    /// the current game phase, which it creates using only the serialized GameStates
    /// sent by the server so mutating this GamePhase does not affect the server.
    phase: GamePhase,

    /// Used to determine which moves or placements the player should take.
    strategy: Box<dyn Strategy>,
}

impl InHousePlayer {
    /// Creates a new AI player using the given streams.
    pub fn new(strategy: Box<dyn Strategy>) -> InHousePlayer {
        InHousePlayer { output_stream: "".to_string(), strategy, phase: GamePhase::Starting }
    }

    /// Helper to create a player with the zigzag minmax strategy.
    pub fn with_zigzag_minmax_strategy() -> InHousePlayer {
        InHousePlayer {
            output_stream: "".to_string(),
            strategy: Box::new(ZigZagMinMaxStrategy),
            phase: GamePhase::Starting
        }
    }

    /// Take a turn by sending a message to the output stream. The contents of the
    /// message depend on the current GamePhase and what the strategy dictates to do
    /// for that phase. For Starting and Done phases, the player can do nothing.
    pub fn take_turn(&mut self) {
        match &mut self.phase {
            // TODO: Should we panic when trying to take a turn in the Starting/Done phases?
            GamePhase::Starting => (),
            GamePhase::PlacingPenguins(gamestate) => {
                let placement = self.strategy.find_placement(gamestate);
                self.send_place_penguin_message(placement).unwrap();
            },
            GamePhase::MovingPenguins(gametree) => {
                let move_ = self.strategy.find_move(gametree);
                self.send_move_penguin_message(move_).unwrap();
            },
            GamePhase::Done(_) => (),
        }
    }

    /// Send a PlacePenguin message to the game server via the given TCP stream
    /// telling it to place one of this player's penguins. The game server will
    /// determine which player sent the message based off their TCP connection info.
    /// If it is not currently that player's turn, or the placement is otherwise
    /// invalid, then they will be kicked from the game and their penguins will
    /// be removed from the board.
    pub fn send_place_penguin_message(&mut self, placement: Placement) -> Result<usize, std::io::Error> { 
        self.output_stream = serde_json::to_string(&Action::PlacePenguin(placement))?;
        Ok(self.output_stream.len())
    }

    /// Send a MovePenguin message over tcp to tell the server to move a given
    /// penguin to the given destination tile. If the penguin has not yet been
    /// placed, does not belong to the current player, or the move itself is
    /// oherwise invalid, the player will be kicked from the game.
    /// 
    /// The game server will determine
    /// which player sent the message based off their TCP connection info.
    /// If it is not currently that player's turn then they will be kicked
    /// from the game and their penguins will be removed from the board.
    pub fn send_move_penguin_message(&mut self, move_: Move) -> Result<usize, std::io::Error> { 
        self.output_stream = serde_json::to_string(&Action::MovePenguin(move_))?;
        Ok(self.output_stream.len())
    }

    /// Block until the server sends a game state at the start of the next turn,
    /// then returns the GameState once one is received.
    /// 
    /// A state will get sent from the server any time an action is performed
    /// by a player that changes the game state (placing a penguin, moving a penguin).
    /// This state is automatically sent to every player and it is the player's job
    /// to recieve the gamestate via receive_gamestate()
    pub fn receive_gamestate(&mut self, bytes: &[u8]) {
        let state = serde_json::from_slice(bytes).unwrap();
        self.phase.update_from_gamestate(state);
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::tile::TileId;
    use crate::common::penguin::PenguinId;
    use crate::client::strategy::ZigZagMinMaxStrategy;

    #[test]
    fn test_send_place_penguin_message() {
        let mut player = InHousePlayer::new(Box::new(ZigZagMinMaxStrategy));

        player.send_place_penguin_message(Placement::new(TileId(1))).unwrap();
        let buffer = &player.output_stream;

        assert_eq!(*buffer, String::from("{\"PlacePenguin\":{\"tile_id\":1}}"));

        player.send_place_penguin_message(Placement::new(TileId(2))).unwrap();
        let buffer = &player.output_stream;

        assert_eq!(*buffer, String::from(
            "{\"PlacePenguin\":{\"tile_id\":2}}"
        ));
    }

    #[test]
    fn test_send_move_penguin_message() {
        let mut player = InHousePlayer::new(Box::new(ZigZagMinMaxStrategy));

        player.send_move_penguin_message(Move::new(PenguinId(1), TileId(1))).unwrap();
        let buffer = &player.output_stream;

        assert_eq!(*buffer, String::from(
            "{\"MovePenguin\":{\"penguin_id\":1,\"tile_id\":1}}"
        ));

        player.send_move_penguin_message(Move::new(PenguinId(2), TileId(2))).unwrap();
        let buffer = &player.output_stream;

        assert_eq!(*buffer, String::from(
            "{\"MovePenguin\":{\"penguin_id\":2,\"tile_id\":2}}"
        ));
    }
}
