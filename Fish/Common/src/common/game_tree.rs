//! This file contains code that represents the GameState at any point
//! during the game, in a lazily-evaluated tree structure.
use crate::common::gamestate::GameState;
use crate::common::action::Move;
use std::collections::HashMap;

/// Represents an entire game of Fish, starting from the given GameState
/// passed to GameTree::new.
/// Takes the form of a tree structure with the nodes being Turns,
/// leaves being Ends, and branches being the valid_moves mappings.
/// Each node stores the GameState representing the data about the
/// game at that point in time.
/// Uses lazy evaluation to avoid storing the entire data structure
/// in memory. See the LazyGameTree struct for more info.
/// 
/// Note that there is no case when a player is stuck; we simply
/// skip their turn if they have no moves and move
/// to the next Turn state.
#[derive(Debug)]
pub enum GameTree {
    Turn { state: GameState, valid_moves: HashMap<Move, LazyGameTree> },
    End(GameState),
}

impl GameTree {
    /// Initialize a GameTree from the given initial GameState.
    /// The given state does not have to be the start of a game -
    /// it is allowed to be any valid game state. It is referred to
    /// as the initial state because the generated tree will start from
    /// that state with links to each potential subsequent state, but
    /// not any previous states.
    pub fn new(initial_state: &GameState) -> GameTree {
        assert!(initial_state.all_penguins_are_placed());

        let valid_moves = initial_state.get_valid_moves();
        if valid_moves.is_empty() {
            GameTree::End(initial_state.clone())
        } else {
            let valid_moves = valid_moves.into_iter().map(|move_| {
                let lazy_game = LazyGameTree::from_move(&move_, initial_state);
                (move_, lazy_game)
            }).collect();

            GameTree::Turn {
                state: initial_state.clone(),
                valid_moves,
            }
        }
    }

    /// Returns a shared reference to the GameState of the current node of the GameTree
    pub fn get_state(&self) -> &GameState {
        match self {
            GameTree::Turn { state, .. } => state,
            GameTree::End(state) => state,
        }
    }

    /// Returns a mutable reference to the GameState of the current node of the GameTree
    pub fn get_state_mut(&mut self) -> &mut GameState {
        match self {
            GameTree::Turn { state, .. } => state,
            GameTree::End(state) => state,
        }
    }

    /// Returns the GameState of the current node of the GameTree
    pub fn take_state(self) -> GameState {
        match self {
            GameTree::Turn { state, .. } => state,
            GameTree::End(state) => state,
        }
    }

    /// Returns the `GameTree` that would be produced as a result of taking the given Move.
    /// If the move is invalid (not in valid_moves or self is `End`) then None is returned
    pub fn get_game_after_move(&mut self, move_: Move) -> Option<&mut GameTree> {
        match self {
            GameTree::Turn { valid_moves, .. } => {
                valid_moves.get_mut(&move_).map(|lazy_game| lazy_game.get_evaluated())
            },
            GameTree::End(_) => None,
        }
    }

    /// Returns the `GameTree` that would be produced as a result of taking the given Move.
    /// If the move is invalid (not in valid_moves or self is `End`) then None is returned
    pub fn take_game_after_move(self, move_: Move) -> Option<GameTree> {
        match self {
            GameTree::Turn { mut valid_moves, .. } => {
                valid_moves.remove(&move_).map(|lazy_game| lazy_game.evaluate())
            },
            GameTree::End(_) => None,
        }
    }

    /// Applies a function to the GameTree for every valid move, returning
    /// a HashMap of the same moves mapped to their new results
    pub fn map<T, F>(&mut self, mut f: F) -> HashMap<Move, T>
        where F: FnMut(&mut GameTree) -> T
    {
        match self {
            GameTree::Turn { valid_moves, .. } => {
                valid_moves.iter_mut().map(|(move_, lazy_game)| {
                    let game = lazy_game.get_evaluated();
                    (move_.clone(), f(game))
                }).collect()
            },
            GameTree::End(_) => HashMap::new(),
        }
    }

    pub fn is_game_over(&self) -> bool {
        match self {
            GameTree::Turn { .. } => false,
            GameTree::End(_) => true,
        }
    }
}

/// A LazyGameTree is either an already evaluted GameTree or
/// is an Unevaluated thunk that can be evaluated to return a GameTree.
/// Since Games are stored as recursive trees in memory keeping
/// the branches of each GameTree::Turn as LazyGameTree::Unevaluated saves
/// us from allocating an exponential amount of memory for every
/// possible GameState. 
pub enum LazyGameTree {
    Evaluated(GameTree),
    Unevaluated(Box<dyn FnMut() -> GameTree>),
}

impl LazyGameTree {
    /// Retrieves the GameTree from this LazyGameTree,
    /// evaluating this LazyGameTree if it hasn't already been
    pub fn get_evaluated(&mut self) -> &mut GameTree {
        match self {
            LazyGameTree::Evaluated(game) => game,
            LazyGameTree::Unevaluated(thunk) => {
                let game = thunk();
                *self = LazyGameTree::Evaluated(game);
                self.get_evaluated()
            },
        }
    }

    pub fn evaluate(self) -> GameTree {
        match self {
            LazyGameTree::Evaluated(game) => game,
            LazyGameTree::Unevaluated(mut thunk) => thunk(),
        }
    }

    /// Create a Unevaluated LazyGameTree from the given state
    /// and the move to take to advance that state. The passed in
    /// move must be valid for the given game state.
    fn from_move(move_: &Move, state: &GameState) -> LazyGameTree {
        let mut state = state.clone();
        let move_ = move_.clone();
        LazyGameTree::Unevaluated(Box::new(move || {
            state.move_avatar_for_current_player(move_)
                .expect(&format!("Invalid move for the given GameState passed to LazyGameTree::from_move.\
                \nMove: {:?}\nGameState: {:?}", move_, state));

            GameTree::new(&state)
        }))
    }
}

impl std::fmt::Debug for LazyGameTree {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        match self {
            LazyGameTree::Evaluated(game) => write!(f, "Evaluated({:?})", game),
            LazyGameTree::Unevaluated(_) => write!(f, "Unevaluated(_)"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::server::strategy::tests::take_zigzag_placement;

    // Starts a game with a 3 row, 5 column board and all penguins placed.
    fn start_game() -> GameTree {
        let mut state = GameState::with_default_board(5, 3, 2);

        while !state.all_penguins_are_placed() {
            take_zigzag_placement(&mut state);
        }

        GameTree::new(&state)
    }

    fn get_expected_valid_moves(game: &GameTree) -> Vec<Move> {
        let mut expected_valid_moves = vec![];
        let state = game.get_state();

        let occupied_tiles = state.get_occupied_tiles();
        for penguin in state.current_player().penguins.iter() {
            let current_tile = state.get_tile(penguin.tile_id.unwrap()).unwrap();
            for tile in current_tile.all_reachable_tiles(&state.board, &occupied_tiles) {
                expected_valid_moves.push(Move::new(current_tile.tile_id, tile.tile_id))
            }
        }

        expected_valid_moves
    }

    #[test]
    fn test_new() {
        // valid_moves generated correctly
        //    - have expected moves, check if same as generated
        // starting gamestate is same as one passed to new
        let game = start_game();
        let mut valid_moves = game.get_state().get_valid_moves();
        let mut expected_valid_moves = get_expected_valid_moves(&game);

        expected_valid_moves.sort();
        valid_moves.sort();

        assert_eq!(expected_valid_moves, valid_moves);
    }

    #[test]
    fn is_initially_unevaluated() {
        let game = start_game();
        match game {
            GameTree::Turn { valid_moves, .. } => {
                // Assert all the branches to the tree are initially Unevaluated
                assert!(valid_moves.iter().all(|(_, lazy_game)| {
                    match lazy_game {
                        LazyGameTree::Evaluated(_) => false,
                        LazyGameTree::Unevaluated(_) => true,
                    }
                }));
            },
            GameTree::End(_) => unreachable!("start_game should never return a finished game"),
        }
    }

    #[test]
    fn test_get_game_after_move() {
        let mut initial_game = start_game();

        // record initial moves and the identity of the player whose turn it is
        let mut initial_valid_moves = initial_game.get_state().get_valid_moves();
        let initial_turn = initial_game.get_state().current_turn;

        let game_after_move = initial_game.get_game_after_move(initial_valid_moves[0]).unwrap(); // make a move

        // record new moves and the identity of the player whose turn it now is
        let mut valid_moves = game_after_move.get_state().get_valid_moves();
        let current_turn = game_after_move.get_state().current_turn;
        let mut expected_valid_moves = get_expected_valid_moves(&game_after_move);

        initial_valid_moves.sort();
        valid_moves.sort();
        expected_valid_moves.sort();

        assert_ne!(initial_turn, current_turn); // turn has changed
        assert_ne!(initial_valid_moves, valid_moves); // valid moves have changed
        assert_eq!(valid_moves, expected_valid_moves); // new valid moves are correct
    }

    #[test]
    fn test_map() {
        let mut game = start_game();

        // Map is_game_over across each state and assert that each value is the
        // same as if we performed the given move then checked is_game_over for
        // the new game state after the move
        let winning_moves = game.map(|game_after_move| game_after_move.is_game_over());
        for (&move_, &game_over) in winning_moves.iter() {
            // Clone the current state, then move the avatar and manually
            // apply the is_game_over function to emulate map's behaviour.
            let mut state_after_move = game.get_state().clone();
            state_after_move.move_avatar_for_current_player(move_);
            assert_eq!(state_after_move.is_game_over(), game_over);
        }

        // ensure map produces a result for every game
        match &game {
            GameTree::Turn { valid_moves, .. } => assert_eq!(winning_moves.len(), valid_moves.len()),
            GameTree::End(_) => unreachable!("start_game should return an in-progress game"),
        }
    }
}
