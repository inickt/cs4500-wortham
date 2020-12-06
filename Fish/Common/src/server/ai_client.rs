//! This file contains the implementation for an in-house AI player
//! for the Fish game.
use crate::common::gamestate::GameState;
use crate::common::game_tree::GameTree;
use crate::common::player::PlayerColor;
use crate::common::action::{ Placement, Move, PlayerMove};
use crate::server::strategy::{ Strategy, ZigZagMinMaxStrategy };
use crate::server::client::Client;


/// Represents the in-house AI client for the Fish game.
/// This client is responsible for using their strategy
/// to determine what action to take on their turn.
pub struct AIClient {
    strategy: Box<dyn Strategy>,
}

impl AIClient {
    /// Creates a new AI client using the given streams.
    pub fn new(strategy: Box<dyn Strategy>) -> AIClient {
        AIClient { strategy }
    }

    /// Helper to create a client with the zigzag minmax strategy.
    pub fn with_zigzag_minmax_strategy() -> AIClient {
        AIClient { strategy: Box::new(ZigZagMinMaxStrategy) }
    }
}

impl Client for AIClient {
    fn tournament_starting(&mut self) -> Option<()> {
        Some(())
    }

    fn tournament_ending(&mut self, _won: bool) -> Option<()> {
        Some(())
    }

    fn initialize_game(&mut self, _initial_gamestate: &GameState, _player_color: PlayerColor) -> Option<()> {
        Some(())
    }

    fn get_placement(&mut self, gamestate: &GameState) -> Option<Placement> {
        Some(self.strategy.find_placement(gamestate))
    }

    fn get_move(&mut self, gamestate: &GameState, _previous: &[PlayerMove]) -> Option<Move> {
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
    use crate::server::strategy::{ tests::take_zigzag_placement, ZigZagMinMaxStrategy };

    #[test]
    fn test_take_turn_placement() {
        let mut player = AIClient::new(Box::new(ZigZagMinMaxStrategy));

        let state = GameState::with_default_board(3, 5, 2);
        assert_eq!(player.get_placement(&state), Some(Placement { tile_id: TileId(0) }));
    }

    #[test]
    fn test_take_turn_move() {
        let mut player = AIClient::new(Box::new(ZigZagMinMaxStrategy));

        let mut state = GameState::with_default_board(3, 5, 2);

        while !state.all_penguins_are_placed() {
            take_zigzag_placement(&mut state);
        }

        let action = player.get_move(&state, &[]);
        assert_eq!(action.unwrap().to, TileId(2));
    }
}
