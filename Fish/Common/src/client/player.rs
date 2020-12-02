//! This file contains the implementation for an in-house AI player
//! for the Fish game.
use crate::common::gamestate::GameState;
use crate::common::game_tree::GameTree;
use crate::common::player::PlayerColor;
use crate::common::action::{Action, Placement, Move, PlayerMove};
use crate::client::strategy::{ Strategy, ZigZagMinMaxStrategy };
use crate::common::gamephase::GamePhase;
use crate::server::message::*;


pub trait PlayerInterface {
    fn tournament_starting(&mut self) -> Option<()>;
    fn tournament_ending(&mut self, won: bool) -> Option<()>;

    fn initialize_game(&mut self, initial_gamestate: &GameState, player_color: PlayerColor) -> Option<()>;
    fn get_placement(&mut self, gamestate: &GameState) -> Option<Placement>;
    fn get_move(&mut self, gamestate: &GameState, previous: &[PlayerMove]) -> Option<Move>;
}

/// Represents the in-house AI player for the Fish game.
/// This player holds their own GamePhase and is responsible for using their strategy
/// to determine what action to take on their turn.
pub struct InHousePlayer {
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
        InHousePlayer { strategy, phase: GamePhase::Starting }
    }

    /// Helper to create a player with the zigzag minmax strategy.
    pub fn with_zigzag_minmax_strategy() -> InHousePlayer {
        InHousePlayer {
            strategy: Box::new(ZigZagMinMaxStrategy),
            phase: GamePhase::Starting
        }
    }

    /// Take a turn by sending a message to the output stream. The contents of the
    /// message depend on the current GamePhase and what the strategy dictates to do
    /// for that phase. For Starting and Done phases, the player can do nothing.
    pub fn take_turn(&mut self) -> Action {
        match &mut self.phase {
            GamePhase::Starting => panic!("Called InHousePlayer::take_turn in the starting phase"),
            GamePhase::PlacingPenguins(gamestate) => {
                Action::PlacePenguin(self.strategy.find_placement(gamestate))
            },
            GamePhase::MovingPenguins(gametree) => {
                Action::MovePenguin(self.strategy.find_move(gametree))
            },
            GamePhase::Done(_) => panic!("Called InHousePlayer::take_turn in the done phase"),
        }
    }

    /// Block until the server sends a game state at the start of the next turn,
    /// then returns the GameState once one is received.
    /// 
    /// A state will get sent from the server any time an action is performed
    /// by a player that changes the game state (placing a penguin, moving a penguin).
    /// This state is automatically sent to every player and it is the player's job
    /// to recieve the gamestate via receive_gamestate()
    pub fn receive_message(&mut self, bytes: &[u8]) {
        match serde_json::from_slice(bytes) {
            Ok(ServerToClientMessage::Setup(state)) => self.update_from_gamestate(state.0),
            Ok(ServerToClientMessage::TakeTurn(state, _)) => self.update_from_gamestate(state),
            Ok(_) => println!("Parsed different message"),
            Err(err) => println!("Failed to accept message!\n{}\n err is {}", String::from_utf8_lossy(bytes), err),
        }
    }

    /// Mutate the current GameState of self.phase to the given
    /// game state described by new_state.
    fn update_from_gamestate(&mut self, new_state: JSONGameState) {
        let new_state = new_state.to_common_game_state();
        self.phase.update_from_gamestate(new_state);
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::tile::TileId;
    use crate::common::action::Placement;
    use crate::common::gamestate::GameState;
    use crate::client::strategy::{ tests::take_zigzag_placement, ZigZagMinMaxStrategy };

    #[test]
    fn test_take_turn_placement() {
        let mut player = InHousePlayer::new(Box::new(ZigZagMinMaxStrategy));

        let state = GameState::with_default_board(3, 5, 2);
        let message = setup_message(&state);
        player.receive_message(message.as_bytes());

        assert_eq!(player.take_turn(), Action::PlacePenguin(Placement { tile_id: TileId(0) }));
    }

    #[test]
    fn test_take_turn_move() {
        let mut player = InHousePlayer::new(Box::new(ZigZagMinMaxStrategy));

        let mut state = GameState::with_default_board(3, 5, 2);

        for _ in 0 .. state.all_penguins().len() {
            take_zigzag_placement(&mut state); // place all penguins using the zigzag method
        }

        let message = take_turn_message(&state, &[]);
        player.receive_message(message.as_bytes());

        let action = player.take_turn();
        assert_eq!(action.as_move().unwrap().tile_id, TileId(2));
    }

    #[test]
    fn test_receive_setup_message() {
        let mut player = InHousePlayer::new(Box::new(ZigZagMinMaxStrategy));
        let state = GameState::with_default_board(3, 5, 2);

        let message = setup_message(&state);
        player.receive_message(message.as_bytes());

        assert_eq!(player.phase.take_state(), state);
    }
}
