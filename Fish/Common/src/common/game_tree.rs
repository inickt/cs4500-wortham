use crate::common::gamestate::GameState;
use crate::common::penguin::PenguinId;
use crate::common::tile::TileId;
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

#[derive(Clone, PartialEq, Eq, Hash)]
struct Move {
    penguin_id: PenguinId,
    tile_id: TileId,
}

impl Game {
    fn new(initial_state: &GameState) -> Game {
        // let possible_moves = initial_state.
        unimplemented!();
    }

    fn get_state(&self) -> &GameState {
        match self {
            Game::Turn { state, .. } => state,
            Game::End(state) => state,
        }
    }

    /// Returns the `Game` that would be produced as a result of taking the given Moves in order.
    /// If the move is invalid (not in valid_moves or self is `End`) then None is returned
    fn get_game_after_move(&mut self, move_: Move) -> Option<&Game> {
        match self {
            Game::Turn { state, valid_moves } => {
                valid_moves.get_mut(&move_).map(|lazy_game| lazy_game.get_or_evaluate())
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
                    let game = lazy_game.get_or_evaluate();
                    (key.clone(), f(game.get_state()))
                }).collect()
            },
            Game::End(_) => HashMap::new(),
        }
    }
}

enum LazyGame {
    Evaluated(Game),
    Unevaluated(Box<dyn Fn() -> Game>),
}

impl LazyGame {
    fn get_or_evaluate(&mut self) -> &Game {
        match self {
            LazyGame::Evaluated(game) => game,
            LazyGame::Unevaluated(thunk) => {
                let game = thunk();
                *self = LazyGame::Evaluated(game);
                self.get_or_evaluate()
            },
        }
    }
}