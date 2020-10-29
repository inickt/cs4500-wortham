/// This file contains code representing different strategies used by
/// the player when playing the game.
use crate::common::gamestate::GameState;
use crate::common::game_tree::Game;
use crate::common::player::PlayerId;
use crate::common::action::Move;
use crate::common::util::{ all_min_by_key, all_max_by_key };

use std::collections::HashMap;

/// Places a penguin for the player whose turn it currently is 
/// at the next available spot on the game board, according to
/// the following zig-zag algorithm:
/// 1. Start at row 0, col 0
/// 2. Search left -> right in the current row
/// 3. If there's an empty spot, place a penguin there and exit
/// 4. If not, go to the next row and go back to step 2
/// 
/// The function assumes the board will have enough open spots to hold
/// all the penguins of its players, i.e. there will always be an
/// open spot.
/// 
/// This function panics if the current player has no unplaced penguins.
pub fn place_penguin_zigzag(state: &mut GameState) {
    let player = state.current_player();
    let player_id = player.player_id;
    let penguin_id = player.get_unplaced_penguin_id().expect("All penguins are already placed");
    let occupied_tiles = state.get_occupied_tiles();

    for row in 0 .. state.board.height {
        for col in 0 .. state.board.width {
            let tile_id = state.board.get_tile_id(col, row).unwrap();
            if !occupied_tiles.contains(&tile_id) {
                state.place_avatar_for_player(player_id, penguin_id, tile_id);
                return;
            }
        }
    }

    unreachable!("place_penguin_zigzag: cannot place penguin, all board positions are filled")
}

/// Makes a move to maximize the current player's score after looking ahead
/// a given number of turns, assuming that other players will attempt to minimize
/// the current player's score.
/// 
/// Panics if the game is already over.
pub fn move_penguin_minmax(state: &mut GameState, lookahead: usize) {
    let mut game = Game::new(state);
    let (_, moves) = find_best_score_and_moves(&mut game, state.current_turn, lookahead);

    let move_to_take = *moves.last().expect("The game is over, there are no valid moves!");
    state.move_avatar_for_current_player(move_to_take).unwrap();
}

/// Traverse the Game tree to find a set of moves that maximizes the score of the given player,
/// assuming all opponents want to minimize the player's score.
/// 
/// Returns the score of the given player and a Vec of all moves of each player in reverse order
///     i.e. the first move taken in the path occurs last in the Vec
/// 
/// Termination: lookahead decreases by 1 each time the given player takes a turn. Since the
///   turn order will always come back to the same player eventually (unless the game ends), this is
///   continuously decreasing. The function terminates when either lookahead reaches 0 or a Game::End\
///   node is given, whichever comes first.
/// 
/// See find_best_move for the specific algorithm used to select the best move.
fn find_best_score_and_moves(game: &mut Game, player: PlayerId, lookahead: usize) -> (usize, Vec<Move>) {
    let is_players_turn = game.get_state().current_turn == player;

    if game.is_game_over() || (lookahead == 0 && is_players_turn) {
        (game.get_state().player_score(player), vec![])
    } else {
        // Lookahead is counted in rounds where every player takes a turn,
        // so only decrease it when the given player takes a turn.
        let lookahead = lookahead - if is_players_turn { 1 } else { 0 };

        // Recurse first, getting the expected states after each possible move the current player can take
        // assuming the given player maximizes their score and all opponents minimize it.
        let possible_moves = game.map(|state| {
            let mut game_after_move = Game::new(state);
            find_best_score_and_moves(&mut game_after_move, player, lookahead)
        });

        // Maximize the score for the given player if it's their turn, otherwise take the move that minimizes it
        let (new_move, (score, mut move_history)) = find_best_move(game.get_state(), is_players_turn, possible_moves);
        move_history.push(new_move);
        (score, move_history)
    }
}

/// If it is the given player's turn, filter the moves that maximizes their score.
/// Otherwise, filter the moves that minimizes their score.
/// 
/// Filters by the minimum starting position then the minimum ending position if needed to tie break
/// multiple equally-scored moves.
/// 
/// Returns the (key, value) pair of the given hashmap that represents the best turn following the rules above.
fn find_best_move(state: &GameState, is_players_turn: bool, moves: HashMap<Move, (usize, Vec<Move>)>) -> (Move, (usize, Vec<Move>)) {
    let moves = if is_players_turn {
        all_max_by_key(moves.into_iter(), |(_, (score, _))| *score)
    } else {
        all_min_by_key(moves.into_iter(), |(_, (score, _))| *score)
    };

    // If we still have a tie, settle it by the penguin's position then the destination position in that  order
    let moves = all_min_by_key(moves, |(move_, _)| state.get_penguin_tile_position(move_.penguin_id).unwrap());
    let mut moves = all_min_by_key(moves, |(move_, _)| state.board.get_tile_position(move_.tile_id));

    moves.nth(0).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::tile::TileId;

    #[test]
    fn test_place_penguin_zigzag() {
        let mut state = GameState::with_default_board(3, 5, 2);

        let penguins_count = state.all_penguins().len();
        let first_player_id = state.current_player().player_id;

        let occupied_tiles_before_place = state.get_occupied_tiles();
        assert_eq!(occupied_tiles_before_place.len(), 0);

        let mut penguins_placed = 0;

        for row in 0 .. state.board.height {
            for col in 0 .. state.board.width {
                if penguins_placed >= penguins_count {
                    break; // stop iterating through potential locations if we've placed them all
                }

                let prev_player_id = state.current_player().player_id; // record prev player_id
                let prev_occupied_tiles = state.get_occupied_tiles(); // record prev tiles w/ penguins

                place_penguin_zigzag(&mut state); // place the penguin and count it
                penguins_placed += 1;

                let post_occupied_tiles = state.get_occupied_tiles();

                // are there still the right amount of penguins placed?
                assert_eq!(post_occupied_tiles.len(), penguins_placed);

                let new_occupied_tiles = post_occupied_tiles.difference(&prev_occupied_tiles)
                    .collect::<Vec<&TileId>>();
                let placed_tile_id = new_occupied_tiles.first().unwrap();

                // was the new penguin placed in the right spot?
                assert_eq!(state.board.get_tile_position(**placed_tile_id), (col, row).into());

                // did the turn and player update?
                assert_ne!(state.current_player().player_id, prev_player_id);
            }

            if penguins_placed >= penguins_count {
                break; // stop iterating through potential locations if we've placed them all
                       // (must break from outer loop as well)
            }
        }

        // are we back at the first player's turn?
        // This should always be the case since each player should have an equal number of penguins
        assert_eq!(first_player_id, state.current_player().player_id);

    }

    /// This test assures that the algorithm will pick the best move for a one-turn lookahead.
    /// Since this board has 3 fish on each tile, there will be many such moves. The test also
    /// ensures that the "tiebreaker" criteria of lowest row and column within that row are met.
    #[test]
    fn test_move_penguin_minmax() {
        // 0     4     7    10     13
        //    1     5     8     11    14
        // 2     6     9    12     15
        // has 3 fish on each tile
        let mut state = GameState::with_default_board(3, 5, 2);

        for _ in 0 .. state.all_penguins().len() {
            place_penguin_zigzag(&mut state); // place all penguins using the zigzag method
        }

        // placements of penguins (p1 = player1, p2 = player2)
        // 3 fish on each tile
        // p1    p2    p1    p2    p1
        //    p2    p1    p2    11   14
        // 2     6     9     12    15

        // looking ahead 1 turn, the move algorithm should pick the move p1(0,0) -> 2.
        // since all tiles have 3 fish, the algorithm should pick the move with the lowest row,
        // and within that row, the lowest column, since the gain will be 3 for any move.
        let penguin_to_move = state.find_penguin_at_position((0, 0).into()).unwrap().penguin_id;

        move_penguin_minmax(&mut state, 1);
        let new_tile = state.find_penguin(penguin_to_move).unwrap().tile_id.unwrap();
        let new_pos = state.board.get_tile_position(new_tile);
        assert_eq!(new_pos, (0, 2).into());
    }

    /// This test ensures that the algorithm will make winning moves
    /// when looking many turns ahead.
    #[test]
    fn test_move_penguin_minmax_lookahead() {
        let mut state = GameState::with_default_board(3, 5, 2);

        for _ in 0 .. state.all_penguins().len() {
            place_penguin_zigzag(&mut state); // place all penguins using the zigzag method
        }

        // initial placements of penguins (p1 = player1, p2 = player2)
        // 3 fish on each tile
        // p1    p2    p1    p2   p1
        //    p2    p1    p2    3    3
        // 3     3     3     3    3

        // end placements of penguins (p1 = player1, p2 = player2, x = hole)
        // p1 (4) score: 4 tiles captured x 3 fish = 12
        // p2 (5) score: 3 tiles captured x 3 fish = 9
        // x     x     p1    x     x 
        //    p2    p1    x     x    p1
        // p1    p2    p2    p2    x 

        // Looking ahead 20 turns, the move algorithm sees multiple paths to a winning game.
        // It will eventually have to move (x, y) p1(4, 0) to (3, 1). First, it moves p1(0,0) to (0,2).
        // Since this move will happen regardless, it makes it first because it is the move with the lowest
        // column in the lowest row.

        // First move should be (0, 0) to (0, 2)
        let penguin_to_move = state.find_penguin_at_position((0, 0).into()).unwrap().penguin_id;
        move_penguin_minmax(&mut state, 20);
        let new_tile = state.find_penguin(penguin_to_move).unwrap().tile_id.unwrap();
        let new_pos = state.board.get_tile_position(new_tile);
        assert_eq!(new_pos, (0, 2).into());

        // Second move should be player 2 (1, 0) to (1, 2)
        let penguin_to_move = state.find_penguin_at_position((1, 0).into()).unwrap().penguin_id;
        let expected_minimizing_move = Move::new(penguin_to_move, state.board.get_tile_id(1, 2).unwrap());
        state.move_avatar_for_current_player(expected_minimizing_move);

        // Third move should be player 1 (4, 0) to (3, 1)
        // This is the "cornerstone" move of the game, in which player 1 guarantees a win
        // We know now that the algorithm is not simply picking the move with the lowest row and column,
        // because that move would be (2, 0) to (2, 2).
        let penguin_to_move = state.find_penguin_at_position((4, 0).into()).unwrap().penguin_id;
        move_penguin_minmax(&mut state, 20);
        let new_tile = state.find_penguin(penguin_to_move).unwrap().tile_id.unwrap();
        let new_pos = state.board.get_tile_position(new_tile);
        assert_eq!(new_pos, (3, 1).into());
    }
}