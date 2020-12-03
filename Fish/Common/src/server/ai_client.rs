//! This file contains the implementation for an in-house AI player
//! for the Fish game.
use crate::common::gamestate::GameState;
use crate::common::game_tree::GameTree;
use crate::common::player::PlayerColor;
use crate::common::action::{ Placement, Move, PlayerMove};
use crate::server::strategy::{ Strategy, ZigZagMinMaxStrategy };
use crate::common::gamephase::GamePhase;
use crate::server::client::Client;


/// Represents the in-house AI player for the Fish game.
/// This player holds their own GamePhase and is responsible for using their strategy
/// to determine what action to take on their turn.
pub struct AIClient {
    /// Contains the current phase of the game (starting, placing, moving, done),
    /// which also contains either the current GameState or GameTree depending on
    /// if we are in the Placing or Moving phase. This is the player's concept of
    /// the current game phase, which it creates using only the serialized GameStates
    /// sent by the server so mutating this GamePhase does not affect the server.
    phase: GamePhase,

    /// Used to determine which moves or placements the player should take.
    strategy: Box<dyn Strategy>,
}

impl AIClient {
    /// Creates a new AI player using the given streams.
    pub fn new(strategy: Box<dyn Strategy>) -> AIClient {
        AIClient { strategy, phase: GamePhase::Starting }
    }

    /// Helper to create a player with the zigzag minmax strategy.
    pub fn with_zigzag_minmax_strategy() -> AIClient {
        AIClient {
            strategy: Box::new(ZigZagMinMaxStrategy),
            phase: GamePhase::Starting
        }
    }
}

impl Client for AIClient {
    fn tournament_starting(&mut self) -> Option<()> {
        Some(())
    }

    fn tournament_ending(&mut self, won: bool) -> Option<()> {
        Some(())
    }

    fn initialize_game(&mut self, initial_gamestate: &GameState, player_color: PlayerColor) -> Option<()> {
        Some(())
    }

    fn get_placement(&mut self, gamestate: &GameState) -> Option<Placement> {
        Some(self.strategy.find_placement(gamestate))
    }

    fn get_move(&mut self, gamestate: &GameState, previous: &[PlayerMove]) -> Option<Move> {
        let mut gametree = GameTree::new(gamestate);
        Some(self.strategy.find_move(&mut gametree))
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

        while !state.all_penguins_are_placed() {
            take_zigzag_placement(&mut state);
        }

        let message = take_turn_message(&state, &[]);
        player.receive_message(message.as_bytes());

        let action = player.take_turn();
        assert_eq!(action.as_move().unwrap().to, TileId(2));
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
