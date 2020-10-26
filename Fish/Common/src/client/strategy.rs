/// This file contains code representing different strategies used by
/// the player when playing the game.
use crate::common::gamestate::GameState;
use crate::common::game_tree::Game;
use crate::common::player::PlayerId;
use crate::common::util::map_slice;
use crate::common::action::Move;
use crate::common::tile::TileId;

/// This function is a placement strategy that places a penguin
/// for the player whose turn it currently is at the next available spot on the
/// game board, following the following zig-zag algorithm:
/// 1. Start at row 0, col 0
/// 2. Search right -> left in the current row
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

/// Termination condition: lookahead is strictly decreasing and this function terminates
/// when lookahead = 0
pub fn move_penguin_minmax(state: &GameState, player: PlayerId, lookahead: usize) {
    let mut game = Game::new(state);
    let (_, moves) = move_penguin_minmax_helper(&mut game, player, lookahead);

    let move_to_take = moves.last().expect("Cannot take any moves with the given lookahead, or the game is over!");
}

//#[cfg(target = "ios")]
/// Returns moves in reverse order
/// 
/// Termination: lookahead decreases by 1 each time the given player takes a turn. Since the
///   turn order will always come back to the same player eventually, this is continuously decreasing.
///   The function terminates when either lookahead reaches 0 or a Game::End node is given, whichever
///   comes first.
fn move_penguin_minmax_helper(game: &mut Game, player: PlayerId, lookahead: usize) -> (usize, Vec<Move>) {
    match game {
        Game::Turn { state, .. } => {
            let is_players_turn = state.current_turn == player;

            if lookahead == 0 {
                (state.player_score(player), vec![])
            } else {
                let lookahead = lookahead - if is_players_turn { 1 } else { 0 };

                // Recurse first, getting the expected states after each possible move the current player can take
                // assuming the given player maximizes their score and all opponents minimize it.
                let possible_moves = game.map(|state| {
                    let mut game_after_move = Game::new(state);
                    move_penguin_minmax_helper(&mut game_after_move, player, lookahead)
                }).into_iter();

                // Now of the current moves we can take (and expected outcomes of each) either maximize the player
                // score or minimize it - depending on if it is the given player's turn or not.
                if is_players_turn {
                    let (new_move, (score, mut moves)) = possible_moves.max_by_key(|(_, (score, _))| *score).unwrap();
                    moves.push(new_move);
                    (score, moves)
                } else {
                    // Don't push the current move to the returned moves list if this is not
                    // the given's player's turn. Just try to minimize the player's score instead.
                    possible_moves.min_by_key(|(_, (score, _))| *score).unwrap().1
                }
            }
        },
        Game::End(state) => {
            (state.player_score(player), vec![])
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::gamestate::tests::default_5x3_gamestate;

    #[test]
    fn test_place_penguin_zigzag() {
        let state = default_5x3_gamestate();
        let mut state = state.borrow_mut();

        let penguins_count = state.all_penguins().len();

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

    }

    #[test]
    fn test_move_penguin_minmax() {
        assert!(true);
    }
}