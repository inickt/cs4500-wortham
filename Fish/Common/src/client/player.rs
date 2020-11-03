//! This file contains the implementation for an in-house AI player
//! for the Fish game.
use crate::common::action::{ Move, Placement };
use crate::common::gamestate::GameState;
use crate::common::game_tree::GameTree;
use crate::common::tile::TileId;
use crate::client::strategy::Strategy;

use std::io::{ Read, Write };
use std::mem::discriminant;

use serde::{ Serialize, Deserialize };
use serde_json::{ Deserializer, de::IoRead, json };

/// Represents the in-house AI player for the Fish game.
struct InHousePlayer<In: Read, Out: Write, Strat: Strategy> {
    /// Stream from which the player receives data from the referee.
    deserializer: Deserializer<IoRead<In>>,

    /// Stream through which the player may send messages to the referee.
    pub output_stream: Out,
    
    phase: GamePhase,

    /// Used to determine which moves or placements the player should take.
    strategy: Strat,
}

/// Represents the step of the Fish game protocol the game is on currently.
/// You can find the protocol in Fish/Planning/player-protocol.md
enum GamePhase {
    /// The game is beginning, and no actions should be taken.
    Starting,

    /// Players may place penguins, but nothing else.
    /// Uses a GameState since GameTrees do not represent boards with unplaced penguins.
    PlacingPenguins(GameState),

    /// Players may move penguins, but nothing else.
    /// Uses a tree to plan moves ahead, and stores this tree
    /// so that leaves are not recomputed.
    MovingPenguins(GameTree),

    /// The game is over, and the winner(s) are stated in the GameState's winning_players field.
    Done
}

impl GamePhase {
    // Is this GamePhase in the same phase as other?
    pub fn same_phase(&self, other: &GamePhase) -> bool {
        discriminant(self) == discriminant(other)
    }
}

impl Default for GamePhase {
    fn default() -> Self {
        GamePhase::Starting
    }
}

impl<In: Read, Out: Write, Strat: Strategy> InHousePlayer<In, Out, Strat> {
    /// Creates a new AI player using the given streams.
    pub fn new(input_stream: In, output_stream: Out, strategy: Strat) -> InHousePlayer<In, Out, Strat> {
        let deserializer = Deserializer::from_reader(input_stream);
        InHousePlayer { deserializer, output_stream, strategy, phase: GamePhase::Starting }
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
            GamePhase::Done => (),
        }
    }

    /// Send a PlacePenguin message to the game server via the given TCP stream
    /// telling it to place one of this player's penguins. The game server will
    /// determine which player sent the message based off their TCP connection info.
    /// If it is not currently that player's turn, or the placement is otherwise
    /// invalid, then they will be kicked from the game and their penguins will
    /// be removed from the board.
    pub fn send_place_penguin_message(&mut self, placement: Placement) -> Result<usize, std::io::Error> { 
        let message_json = json!({ "type": "PlacePenguin", "tile_id": placement.tile_id.0 });
        let msg = serde_json::to_string(&message_json)?;
        self.output_stream.write(msg.as_bytes())
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
        let message_json = json!({ "type": "MovePenguin", "penguin_id": move_.penguin_id.0, "tile_id": move_.tile_id.0 });
        let msg = serde_json::to_string(&message_json)?;
        self.output_stream.write(msg.as_bytes())
    }

    /// Block until the server sends a game state at the start of the next turn,
    /// then returns the GameState once one is received.
    /// 
    /// A state will get sent from the server any time an action is performed
    /// by a player that changes the game state (placing a penguin, moving a penguin).
    /// This state is automatically sent to every player and it is the player's job
    /// to recieve the gamestate via receive_gamestate()
    /// 
    /// This function returns a GameState, which contains all the information
    /// about the given game. Players may wish to view `src/common/gamestate.rs`
    /// for more information and documentation about how to work with this
    /// struct. Players may also wish to use Game struct to check if their planned
    /// moves are valid, or to peek ahead into the future.
    fn receive_gamestate(&mut self) -> Option<GameState> {
        if self.phase.same_phase(&GamePhase::Done) {
            None
        } else {
            // TODO: Is there a better way to signal errors?
            GameState::deserialize(&mut self.deserializer).ok()
        }
    }

    pub fn update_gamestate_from_server(&mut self) {
        if let Some(gamestate) = self.receive_gamestate() {
            if !gamestate.all_penguins_are_placed() {
                self.phase = GamePhase::PlacingPenguins(gamestate);
            } else if !gamestate.is_game_over() {
                self.update_gametree_position(&gamestate);
            } else {
                self.phase = GamePhase::Done;
            }
        }
    }

    /// Given self.phase is GamePhase::MovingPenguins(tree), search for a
    /// matching gamestate within the tree and mutate self.phase to contain
    /// that child tree. This has the effect of moving us forward by 1 player
    /// turn in the current GameTree.
    fn update_gametree_position(&mut self, child_state: &GameState) {
        // Move out of the current phase so we can get each child tree by value later on.
        let current_phase = std::mem::take(&mut self.phase);

        if let GamePhase::MovingPenguins(GameTree::Turn { valid_moves, .. }) = current_phase {
            for (_, game_after_move) in valid_moves {
                let game_after_move = game_after_move.evaluate();
                if game_after_move.get_state() == child_state {
                    self.phase = GamePhase::MovingPenguins(game_after_move);
                    return;
                }
            }
        }
        self.phase = GamePhase::MovingPenguins(GameTree::new(child_state));
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::penguin::PenguinId;
    use crate::client::strategy::ZigZagMinMaxStrategy;

    fn buf_to_string(buf: &Vec<u8>) -> String {
        std::str::from_utf8(buf.as_slice()).unwrap().into()
    }

    #[test]
    fn test_send_place_penguin_message() {
        let buffer = Vec::new();
        let input  = "".as_bytes();
        
        let mut player = InHousePlayer::new(input, buffer, ZigZagMinMaxStrategy);

        player.send_place_penguin_message(Placement::new(TileId(1))).unwrap();
        let buffer = &player.output_stream;

        assert_eq!(buf_to_string(buffer), String::from("{\"tile_id\":1,\"type\":\"PlacePenguin\"}"));

        player.send_place_penguin_message(Placement::new(TileId(2))).unwrap();
        let buffer = &player.output_stream;

        assert_eq!(buf_to_string(buffer), String::from(
            "{\"tile_id\":1,\"type\":\"PlacePenguin\"}{\"tile_id\":2,\"type\":\"PlacePenguin\"}"
        ));
    }

    #[test]
    fn test_send_move_penguin_message() {
        let buffer = Vec::new();
        let input  = "".as_bytes();
        
        let mut player = InHousePlayer::new(input, buffer, ZigZagMinMaxStrategy);

        player.send_move_penguin_message(Move::new(PenguinId(1), TileId(1))).unwrap();
        let buffer = &player.output_stream;

        assert_eq!(buf_to_string(buffer), String::from(
            "{\"penguin_id\":1,\"tile_id\":1,\"type\":\"MovePenguin\"}"
        ));

        player.send_move_penguin_message(Move::new(PenguinId(2), TileId(2))).unwrap();
        let buffer = &player.output_stream;

        assert_eq!(buf_to_string(buffer), String::from(
            "{\"penguin_id\":1,\"tile_id\":1,\"type\":\"MovePenguin\"}\
            {\"penguin_id\":2,\"tile_id\":2,\"type\":\"MovePenguin\"}"
        ));
    }
}
