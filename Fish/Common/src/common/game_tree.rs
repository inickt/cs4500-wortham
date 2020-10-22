//! This file contains code that represents the GameState at any point
//! during the game, in a lazily-evaluated tree structure.
use crate::common::gamestate::GameState;
use crate::common::action::Move;
use std::collections::HashMap;

/// Represents the entire game state, including any possible move.
/// Takes the form of a tree structure with the nodes being Turns,
/// leaves being Ends, and branches being the valid_moves mappings.
/// Each node store the GameState representing the data about the
/// game at that point in time.
/// Uses lazy evaluation to avoid storing the entire data structure
/// in memory. See the LazyGame struct for more info.
enum Game {
    Turn { state: GameState, valid_moves: HashMap<Move, LazyGame> },
    End(GameState),
}

impl Game {
    /// Initialize a Game tree from the given initial GameState.
    /// The given state does not have to be the start of a game -
    /// it is allowed to be any valid game state. It is referred to
    /// as the initial state because the generated tree will start from
    /// that state with links to each potential subsequent state, but
    /// not any previous states.
    fn new(initial_state: &GameState) -> Game {
        // Assert all penguins are already placed on the board
        assert!(initial_state.all_penguins().iter()
            .all(|(_, penguin_id)| initial_state.find_penguin(*penguin_id).unwrap().is_placed()));

        let valid_moves = initial_state.get_valid_moves();
        if valid_moves.is_empty() {
            Game::End(initial_state.clone())
        } else {
            let valid_moves = valid_moves.into_iter().map(|move_| {
                let lazy_game = LazyGame::from_move(&move_, initial_state);
                (move_, lazy_game)
            }).collect();

            Game::Turn {
                state: initial_state.clone(),
                valid_moves,
            }
        }
    }

    /// Returns a shared reference to the GameState of the current node of the Game tree
    fn get_state(&self) -> &GameState {
        match self {
            Game::Turn { state, .. } => state,
            Game::End(state) => state,
        }
    }

    /// Returns the `Game` that would be produced as a result of taking the given Move.
    /// If the move is invalid (not in valid_moves or self is `End`) then None is returned
    fn get_game_after_move(&mut self, move_: Move) -> Option<&Game> {
        match self {
            Game::Turn { valid_moves, .. } => {
                valid_moves.get_mut(&move_).map(|lazy_game| lazy_game.get_evaluated())
            },
            Game::End(_) => None,
        }
    }

    /// Applies a function to the Game for every valid move, returning
    /// a HashMap of the same moves mapped to their new results
    fn map<T, F>(&mut self, mut f: F) -> HashMap<Move, T>
        where F: FnMut(&GameState) -> T
    {
        match self {
            Game::Turn { valid_moves, .. } => {
                valid_moves.iter_mut().map(|(key, lazy_game)| {
                    let game = lazy_game.get_evaluated();
                    (key.clone(), f(game.get_state()))
                }).collect()
            },
            Game::End(_) => HashMap::new(),
        }
    }
}

/// A LazyGame is either an already evaluted Game or
/// is an Unevaluated thunk that can be evaluated to return a Game.
/// Since Games are stored as recursive trees in memory keeping
/// the branches of each Game::Turn as LazyGame::Unevaluated saves
/// us from allocating an exponential amount of memory for every
/// possible GameState. 
enum LazyGame {
    Evaluated(Game),
    Unevaluated(Box<dyn FnMut() -> Game>),
}

impl LazyGame {
    /// Retrieves the Game from this LazyGame,
    /// evaluating this LazyGame if it hasn't already been
    fn get_evaluated(&mut self) -> &Game {
        match self {
            LazyGame::Evaluated(game) => game,
            LazyGame::Unevaluated(thunk) => {
                let game = thunk();
                *self = LazyGame::Evaluated(game);
                self.get_evaluated()
            },
        }
    }

    /// Create a Unevaluated LazyGame from the given state
    /// and the move to take to advance that state. The passed in
    /// move must be valid for the given game state.
    fn from_move(move_: &Move, state: &GameState) -> LazyGame {
        let mut state = state.clone();
        let move_ = move_.clone();
        LazyGame::Unevaluated(Box::new(move || {
            state.move_avatar_for_current_player(move_)
                .expect(&format!("Invalid move for the given GameState passed to LazyGame::from_move.\
                \nMove: {:?}\nGameState: {:?}", move_, state));

            Game::new(&state)
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::gamestate::tests::*;

    // Starts a game with a 5x3 board and all penguins placed.
    fn start_game() -> Game {
        let initial_state = default_5x3_gamestate();
        let mut state = initial_state.borrow_mut();

        let mut tile_ids: Vec<_> = state.board.tiles.iter().map(|(tile_id, _)| *tile_id).collect();
        tile_ids.sort();
        tile_ids.reverse();

        for (player_id, penguin_id) in state.all_penguins() {
            let tile_id = tile_ids.pop().unwrap();
            state.place_avatar_without_changing_turn(player_id, penguin_id, tile_id);
        }

        Game::new(&state)
    }

    fn get_expected_valid_moves(game: &Game) -> Vec<Move> {
        let mut expected_valid_moves = vec![];
        let state = game.get_state();

        let occupied_tiles = state.get_occupied_tiles();
        for penguin in state.current_player().penguins.iter() {
            let current_tile = state.get_tile(penguin.tile_id.unwrap()).unwrap();
            for tile in current_tile.all_reachable_tiles(&state.board, &occupied_tiles) {
                expected_valid_moves.push(Move::new(penguin.penguin_id, tile.tile_id))
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
            Game::Turn { valid_moves, .. } => {
                // Assert all the branches to the tree are initially Unevaluated
                assert!(valid_moves.iter().all(|(_, lazy_game)| {
                    match lazy_game {
                        LazyGame::Evaluated(_) => false,
                        LazyGame::Unevaluated(_) => true,
                    }
                }));
            },
            Game::End(_) => unreachable!("start_game should never return a finished game"),
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
        let winning_moves = game.map(|state| state.is_game_over());
        for (&move_, &game_over) in winning_moves.iter() {
            // Clone the current state, then move the avatar and manually
            // apply the is_game_over function to emulate map's behaviour.
            let mut state_after_move = game.get_state().clone();
            state_after_move.move_avatar_for_current_player(move_);
            assert_eq!(state_after_move.is_game_over(), game_over);
        }

        // ensure map produces a result for every game
        match &game {
            Game::Turn { valid_moves, .. } => assert_eq!(winning_moves.len(), valid_moves.len()),
            Game::End(_) => unreachable!("start_game should return an in-progress game"),
        }
    }
}