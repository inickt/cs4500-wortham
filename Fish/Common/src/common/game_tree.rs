use crate::common::gamestate::GameState;
use crate::common::action::Move;
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

impl Game {
    fn new(initial_state: &GameState) -> Game {
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
    Unevaluated(Box<dyn FnMut() -> Game>),
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

    /// Create a Unevaluated LazyGame from the given state
    /// and the move to take to advance that state. The passed in
    /// move must be valid for the given game state.
    fn from_move(move_: &Move, state: &GameState) -> LazyGame {
        let mut state = state.clone();
        let move_ = move_.clone();
        LazyGame::Unevaluated(Box::new(move || {
            let current_player = state.current_turn;
            state.move_avatar_for_player(current_player, move_.penguin_id, move_.tile_id)
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

    // Starts a game with a 4x3 board and all penguins placed.
    fn start_game() -> Game {
        let initial_state = default_4x3_gamestate();
        let mut state = initial_state.borrow_mut();

        let mut tile_ids: Vec<_> = state.board.tiles.iter().map(|(tile_id, _)| *tile_id).collect();

        for (player_id, penguin_id) in state.all_penguins() {
            let tile_id = tile_ids.pop().unwrap();
            state.place_avatar_for_player(player_id, penguin_id, tile_id);
        }

        Game::new(&state)
    }

    #[test]
    fn test_new() {
        // valid_moves generated correctly
        //    - have expected moves, check if same as generated
        // starting gamestate is same as one passed to new
        // p0   p4   8
        //   p1   p5   9
        // p2   p6   10
        //   p3   p7   11
        let game = start_game();
        let valid_moves = game.get_state().get_valid_moves();
        // let p5 = PenguinId(5);
        // let p7 = PenguinId(7);
        // let expected_valid_moves = vec![
        //     Move::new(p5, TileId(9)),
        // ];
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

    }

    #[test]
    fn test_map() {

    }
}