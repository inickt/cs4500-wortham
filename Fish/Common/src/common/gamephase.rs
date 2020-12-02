//! This file contains code representing the phase of the game,
//! either starting, placing penguins, moving penguins, or ended.
use crate::common::gamestate::GameState;
use crate::common::game_tree::GameTree;
use crate::common::player::PlayerId;
use crate::common::action::Move;

/// Represents the step of the Fish game protocol the game is on currently.
/// This struct is necessary because it allows us to represent a Game
/// from conception to finish, using the GameState for the placing-penguins
/// period (which GameTree cannot represent) and GameTree for the moving-penguins
/// period for quick and efficient checks on the GameState.
/// 
/// You can find the protocol in Fish/Planning/player-protocol.md
#[derive(Debug)]
pub enum GamePhase {
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
    Done(GameState),
}

impl GamePhase {
    /// Is this GamePhase Done?
    pub fn is_game_over(&self) -> bool {
        match self {
            GamePhase::Done(_) => true,
            _ => false,
        }
    }

    /// Gets the state of this GamePhase. Panics if called
    /// on a Starting phase that has o state.
    pub fn get_state(&self) -> &GameState {
        match self {
            GamePhase::Starting => panic!("Tried to get the state of a Starting GamePhase."),
            GamePhase::PlacingPenguins(state) => state,
            GamePhase::MovingPenguins(tree) => tree.get_state(),
            GamePhase::Done(state) => state,
        }
    }

    /// Gets the mutable state of this GamePhase. Panics if called
    /// on a Starting phase that has o state.
    pub fn get_state_mut(&mut self) -> &mut GameState {
        match self {
            GamePhase::Starting => panic!("Tried to get the state of a Starting GamePhase."),
            GamePhase::PlacingPenguins(state) => state,
            GamePhase::MovingPenguins(tree) => tree.get_state_mut(),
            GamePhase::Done(state) => state,
        }
    }

    /// Takes the state of this GamePhase. Panics if called
    /// on a Starting phase that has o state.
    pub fn take_state(self) -> GameState {
        match self {
            GamePhase::Starting => panic!("Tried to get the state of a Starting GamePhase."),
            GamePhase::PlacingPenguins(state) => state,
            GamePhase::MovingPenguins(tree) => tree.take_state(),
            GamePhase::Done(state) => state,
        }
    }

    /// Returns whose turn it currently is for the current GamePhase.
    /// This will panic if the game has not started yet.
    pub fn current_turn(&self) -> PlayerId {
        self.get_state().current_turn
    }

    /// Updates this GamePhase to a MovingPenguins(tree), where tree is computed
    /// either by creating a new tree if we don't yet have one, or by searching for a
    /// matching gamestate within the current tree's children, again creating a new
    /// tree if a match is not found. This has the effect of moving us forward by 1 player
    /// turn in the current GameTree.
    fn update_gametree_position(self, child_state: &GameState) -> GamePhase {
        if let GamePhase::MovingPenguins(GameTree::Turn { valid_moves, .. }) = self {
            for (_, game_after_move) in valid_moves {
                let game_after_move = game_after_move.evaluate();
                if game_after_move.get_state() == child_state {
                    return GamePhase::MovingPenguins(game_after_move);
                }
            }
        }
        GamePhase::MovingPenguins(GameTree::new(child_state))
    }

    /// Mutates this GamePhase to match a given GameState, e.g. self becomes
    /// if the GameState has winning players, GamePhase::Done,
    /// if the GameState has unplaced penguins, GamePhase::PlacingPenguins
    /// if the GameState's penguins are placed but the game isn't over, GamePhase::MovingPenguins
    pub fn update_from_gamestate(&mut self, gamestate: GameState) {
        let phase = std::mem::replace(self, GamePhase::Starting);

        *self = if !gamestate.all_penguins_are_placed() {
            GamePhase::PlacingPenguins(gamestate)
        } else if !gamestate.is_game_over() {
            phase.update_gametree_position(&gamestate)
        } else {
            GamePhase::Done(gamestate)
        };
    }

    /// "Perform" a move, mutating the current game phase to the game phase after
    /// the given move. If the given move is not valid, no mutation will be done
    /// and None will be returned instead.
    pub fn try_do_move(&mut self, move_: Move) -> Option<()> {
        // Validate the move first
        match self {
            GamePhase::MovingPenguins(tree) => {
                let result = tree.get_game_after_move(move_);
                if result.is_none() {
                    return None;
                }
            },
            _ => return None,
        }

        let phase = std::mem::replace(self, GamePhase::Starting);

        // Then mutate self to be the game after the given move
        if let GamePhase::MovingPenguins(tree) = phase {
            let game_after_move = tree.take_game_after_move(move_).unwrap();
            *self = GamePhase::MovingPenguins(game_after_move);
        }
        Some(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::strategy::tests::take_zigzag_placement;

    fn place_penguins(state: &mut GameState) {
        while !state.all_penguins_are_placed() {
            take_zigzag_placement(state);
        }
    }

    #[test]
    fn test_update_from_gamestate() {
        let mut state1 = GameState::with_default_board(3, 4, 3);
        place_penguins(&mut state1);

        // create expected state after first move
        let mut state1_after_move = state1.clone();
        let moves = state1_after_move.get_valid_moves();
        let move_ = moves.first().unwrap();
        state1_after_move.move_avatar_for_current_player(*move_);

        let expected_phase = GamePhase::MovingPenguins(GameTree::new(&state1_after_move));

        let mut actual_phase = GamePhase::MovingPenguins(GameTree::new(&state1));
        actual_phase.update_from_gamestate(state1_after_move);

        assert_eq!(actual_phase.get_state(), expected_phase.get_state());
    }
}
