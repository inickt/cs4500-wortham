//! The gamestate module defines the SharedGameState that will be
//! serialized by the server and sent to each client - informing
//! them of any updates to the game. The GameState itself is a
//! shared mutable pointer which in the client is shared between
//! the communication layer (TBD) and the ui layer. It represents
//! the full state of the game at any given point in time.
use crate::common::board::Board;
use crate::common::tile::{ TileId, Tile };
use crate::common::player::{ Player, PlayerId, PlayerColor };
use crate::common::penguin::Penguin;
use crate::common::action::{ Move, Placement };
use crate::common::boardposn::BoardPosn;
use crate::common::util;

use std::collections::HashSet;
use std::rc::Rc;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::fmt;

use serde::{ Serialize, Deserialize };

pub const MIN_PLAYERS_PER_GAME: usize = 2;
pub const MAX_PLAYERS_PER_GAME: usize = 4;

/// Each player receives 6 - player_count penguins to start the game
pub const PENGUIN_FACTOR: usize = 6;

/// Rc<RefCell<T>> gives a copiable, mutable reference to its T
///
/// This SharedGameState is a copiable, mutable pointer to the GameState
/// intended for use in the client since gtk requires ownership of the
/// data passed to its callbacks. Using this, one can pass a copy to each
/// callback and maintain a copy to overwrite with server updates as well.
pub type SharedGameState = Rc<RefCell<GameState>>;

/// The GameState contains the entirety of the current state
/// of the game. It is meant to be serialized into json from the server
/// and sent to each client to deserialize to receive the updated game
/// state each turn. The GameState is rendering-agnostic, so each
/// client is free to render the GameState however it wishes.
///
/// Throughout the gamestate, unique ids are usually used over the objects
/// they refer to so that we can (1) avoid excessive cloning from multiple mutable
/// borrows, (2) serialize the data more easily and (3) enable the creation of
/// external mappings on the server from e.g. PlayerId to some private data if needed.
///
/// - Each player's penguin is contained within the Player struct.
/// - Each penguin struct contains either Some(TileId) if it is currently
///   on a tile or None if it hasn't yet been placed.
/// - Each Player is mapped from their unique PlayerId to the Player struct.
/// - The ordering of players is given by the immutable turn_order. The current
///   turn is given by current_turn which will change each time
///   {place,move}_avatar_for_player is called.
/// - The GameState's current_turn player should never be stuck, unless
///   the game is over, i.e. current_player should always have moves.
///   Players' turns will be skipped in turn_order if they cannot move anymore.
#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GameState {
    pub board: Board,
    pub players: BTreeMap<PlayerId, Player>,
    pub turn_order: Vec<PlayerId>, // INVARIANT: turn_order never changes for a given game, unless a player is kicked
    pub current_turn: PlayerId,
    pub winning_players: Option<Vec<PlayerId>>, // will be None until the game ends
}

impl fmt::Debug for GameState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut board_string = String::new();

        for y in 0..self.board.height {
            if y % 2 == 1 {
                board_string.push_str("   ");
            }
            for x in 0..self.board.width {
                let tile_string = match self.board.get_tile_id(x, y) {
                    Some(id) => {
                        match self.players.values().find(|player|
                            player.penguins.iter().any(|penguin| penguin.tile_id == Some(id)))
                        {
                            Some(player) => {
                                format!("P{}", player.player_id.0)
                            },
                            None => format!("{:2}", id.0),
                        }
                    },
                    None => " x".to_string(),
                };
                board_string.push_str(&tile_string);
                board_string.push_str("    ");
            }
            board_string.push_str("\n");
        }

        writeln!(f, "{}", board_string)?;

        // Write each player, their score, and their penguin positions
        for (player_id, player) in self.players.iter() {
            let current_player_str = if self.current_turn == *player_id { "<- current turn" } else { "" };

            let penguins = util::map_slice(&player.penguins, |penguin| {
                match penguin.tile_id {
                    Some(id) => format!("penguin on tile {}", id.0),
                    None => "unplaced".to_string(),
                }
            }).join(", ");

            writeln!(f, "Player {} - {:?} - score: {} - penguins: [{}] {}",
                player_id.0, player.color, player.score, penguins, current_player_str)?;
        }

        writeln!(f, "")
    }
}

impl GameState {
    /// Create a new GameState with the given board and player_count. Generates new
    /// player ids for the number of players given.
    /// This will panic if player_count is < MIN_PLAYERS_PER_GAME or > MAX_PLAYERS_PER_GAME.
    pub fn new(board: Board, player_count: usize) -> GameState {
        GameState::with_players(board, (0..player_count).map(PlayerId).collect())
    }

    /// Create a new GameState with the given board and turn_order, with the player count equal
    /// to the number of players in turn_order.
    /// This will panic if turn_order.len() is < MIN_PLAYERS_PER_GAME or > MAX_PLAYERS_PER_GAME.
    pub fn with_players(board: Board, turn_order: Vec<PlayerId>) -> GameState {
        // Each player receives 6 - N penguins, where N is the number of players
        let penguins_per_player = PENGUIN_FACTOR - turn_order.len(); 

        let players: BTreeMap<_, _> = turn_order.iter().zip(PlayerColor::iter()).map(|(id, color)| {
            (*id, Player::new(*id, color, penguins_per_player))
        }).collect();

        let current_turn = turn_order[0];

        GameState {
            board,
            players,
            turn_order,
            current_turn,
            winning_players: None,
        }
    }

    /// Creates a new gamestate with a board with a given number of rows and columns,
    /// the given number of players, and no holes.
    pub fn with_default_board(rows: u32, columns: u32, players: usize) -> GameState {
        let board = Board::with_no_holes(rows, columns, 3);
        GameState::new(board, players)
    }

    /// Places an unplaced avatar on a position on the board, and advances the turn. 
    /// Returns Some(()) on success, or None if the player makes an invalid placement.
    /// An invalid placement is one of:
    /// 1. Placement on an invalid position (either out of bounds or a hole)
    /// 2. Placement when the players' avatars are already placed
    /// 3. Placement of a penguin that doesn't belong to the current player
    pub fn place_avatar_for_player(&mut self, player: PlayerId, tile: TileId) -> Option<()> {
        self.place_avatar_without_changing_turn(player, tile)?;
        self.advance_turn();
        Some(())
    }

    /// Place a player's avatar but don't change whose turn it is.
    /// This is useful to more easily place avatars in bulk during testing.
    pub fn place_avatar_without_changing_turn(&mut self, player: PlayerId, tile: TileId) -> Option<()> {
        let occupied_tiles = self.get_occupied_tiles();

        if occupied_tiles.contains(&tile) {
            None
        } else {
            let player = self.players.get_mut(&player)?; 
            player.place_penguin(tile, &self.board)
        }
    }

    /// Places an unplaced avatar on the given placement on the board, and advances the turn. 
    /// Returns Some(()) on success, or None if the player makes an invalid placement.
    /// An invalid placement is one of:
    /// 1. Placement on an invalid position (either out of bounds or a hole)
    /// 2. Placement when the players' avatars are already placed
    /// 
    /// This function will choose which penguin to place for the current player, so it is
    /// impossible for the player to place a penguin that is not theirs.
    pub fn place_avatar_for_current_player(&mut self, placement: Placement) -> Option<()> {
        self.place_avatar_for_player(self.current_turn, placement.tile_id)
    }

    /// Moves a placed avatar from one position to another on the board,
    /// removes the tile that penguin was on, and advances the turn.
    /// Returns Some(()) on success, or None if the player makes an invalid move.
    /// An invalid move is one of:
    /// 1. Move to an invalid position (either out of bounds or hole)
    /// 2. Move when the current avatar has yet to be placed
    /// 3. Move to a tile that is not accessible within a straight line
    ///    of the current tile, with no holes in between.
    /// 4. Move of a penguin that doesn't belong to the player
    pub fn move_avatar_for_player_without_changing_turn(&mut self, player: PlayerId, penguin_start_tile: TileId, destination: TileId) -> Option<()> {
        let occupied = &self.get_occupied_tiles();
        let player = self.players.get_mut(&player)?;
        player.move_penguin(penguin_start_tile, destination, &self.board, occupied)?;
        player.score += self.board.remove_tile(penguin_start_tile);
        Some(())
    }

    /// Helper function which moves an avatar for the player whose turn it currently is.
    pub fn move_avatar_for_current_player(&mut self, move_: Move) -> Option<()> {
        self.move_avatar_for_player_without_changing_turn(self.current_turn, move_.from, move_.to)?;
        self.advance_turn();
        Some(())
    }

    /// Retrieve a tile by its ID. Will return None if the id
    /// does not reference any existing tile. This can happen
    /// if the tile was removed and has become a hole in the board.
    pub fn get_tile(&self, tile_id: TileId) -> Option<&Tile> {
        self.board.tiles.get(&tile_id)
    }

    /// Gets the color of the player whose penguin is on a certain tile
    /// Returns None if there is no penguin on that tile
    pub fn get_color_on_tile(&self, tile_id: TileId) -> Option<PlayerColor> {
        self.players.iter().find_map(|(_, player)| {
            let is_penguin_on_tile = player.penguins.iter().any(|penguin| penguin.tile_id == Some(tile_id));
            if is_penguin_on_tile {
                Some(player.color)
            } else {
                None
            }
        })
    }

    /// Returns true if any player has a penguin they can move,
    /// false if not (and the game is thus over)
    pub fn can_any_player_move_penguin(&self) -> bool {
        let occupied_tiles = self.get_occupied_tiles();
        self.players.iter().any(|(_, player)| player.can_move_a_penguin(&self.board, &occupied_tiles))
    }

    /// Returns true if the given player can move a penguin
    pub fn can_player_move(&self, player: PlayerId) -> bool {
        self.players.get(&player).map_or(false, |player|
            player.can_move_a_penguin(&self.board, &self.get_occupied_tiles()))
    }

    /// Returns the set of tiles on this gamestate's board which have a penguin on them
    pub fn get_occupied_tiles(&self) -> HashSet<TileId> {
        self.players.iter()
            .flat_map(|(_, player)| player.penguins.iter().filter_map(|penguin| penguin.tile_id))
            .collect()
    }

    /// Gets all valid moves for the current GameState,
    /// meaning only move the current player can make
    pub fn get_valid_moves(&self) -> Vec<Move> {
        let occupied_tiles = self.get_occupied_tiles();
        let penguins_to_move = &self.current_player().penguins;

        penguins_to_move.iter().flat_map(|penguin| {
            // penguins in Games are placed, so should always be Some
            let starting_tile_id = penguin.tile_id.expect("A penguin was not placed!"); 
            let starting_tile = self.get_tile(starting_tile_id).expect("A penguin is placed on a hole");

            starting_tile.all_reachable_tiles(&self.board, &occupied_tiles)
                .into_iter()
                .map(move |destination| Move::new(starting_tile_id, destination.tile_id))
        }).collect()
    }

    /// Get a penguin at a position, None if no penguin at that position
    #[allow(dead_code)]
    pub fn find_penguin_at_position(&self, posn: BoardPosn) -> Option<&Penguin> {
        let tile = self.board.get_tile_id(posn.x, posn.y)?;
        self.players.iter().find_map(|(_, player)| {
            player.find_penguin(tile)
        })
    }

    /// Search for the penguin at the given TileId and return it if possible.
    /// Returns None if no penguin at that location was found.
    pub fn find_penguin(&self, tile: TileId) -> Option<&Penguin> {
        self.players.iter().find_map(|(_, player)| {
            player.find_penguin(tile)
        })
    }

    /// Returns the player whose turn it currently is
    pub fn current_player(&self) -> &Player {
        self.players.get(&self.current_turn).unwrap()
    }

    /// Is this game over? We define a game to be "over" if either
    /// some players have won, or there are no players left in the game.
    pub fn is_game_over(&self) -> bool {
        self.winning_players.is_some() || self.players.is_empty()
    }

    #[allow(dead_code)]
    pub fn get_player_by_color_mut(&mut self, color: PlayerColor) -> Option<&mut Player> {
        self.players.iter_mut()
            .find(|(_, player)| player.color == color)
            .map(|(_, player)| player)
    }

    /// Advance the turn of this game to the next player's turn
    /// Will mutate this game's current_turn field.
    /// 
    /// Note that this will skip the turn of any player who cannot
    /// move any penguins. It is an invalid game state for the current
    /// turn to be a player who cannot move any penguins.
    pub fn advance_turn(&mut self) {
        self.advance_turn_index();

        for _ in 0 .. self.players.len() {
            if !self.current_player().has_unplaced_penguins() && self.get_valid_moves().is_empty() {
                self.advance_turn_index()
            } else {
                return;
            }
        }

        // No players have any moves left, find the winning players by those with the maximum score
        self.winning_players = Some(util::all_max_by_key(self.players.iter(), |(_, player)| player.score)
            .map(|(id, _)| *id).collect());
    }

    /// Sets the turn of this game to the next player in order
    fn advance_turn_index(&mut self) {
        if !self.turn_order.is_empty() {
            let current_turn_index = self.turn_order.iter().position(|id| id == &self.current_turn).unwrap();
            let next_turn_index = (current_turn_index + 1) % self.turn_order.len();
            self.current_turn = self.turn_order[next_turn_index];
        }
    }

    /// Sets the turn of the game to the previous player's turn, used when removing a player.
    fn previous_turn_index(&mut self) {
        let current_turn_index = self.turn_order.iter()
            .position(|id| id == &self.current_turn).unwrap();
        let prev_turn_index = if current_turn_index == 0 {
            self.turn_order.len().saturating_sub(1)
        } else {
            (current_turn_index - 1) % self.turn_order.len()
        };
        self.current_turn = self.turn_order[prev_turn_index];
    }

    pub fn player_score(&self, player_id: PlayerId) -> usize {
        self.players[&player_id].score
    }

    /// Returns true if all penguins have a concrete position on the board.
    /// If this is false then we are still in the PlacePenguins phase of the game.
    pub fn all_penguins_are_placed(&self) -> bool {
        self.players.iter().all(|(_, player)| !player.has_unplaced_penguins())
    }

    /// Removes a player and its penguins from this game
    pub fn remove_player(&mut self, player_id: PlayerId) {
        if !self.is_game_over() {
            let should_advance_turn = self.current_turn == player_id;

            // Prepare to advance the current turn past the to-be-removed player
            if should_advance_turn {
                self.previous_turn_index();
            }

            self.players.remove(&player_id);
            self.turn_order.retain(|id| *id != player_id);

            // Now actually advance the turn after the player is removed to properly
            // handle the case where we skip the turns of possibly multiple players
            // whose penguins are all stuck.
            if should_advance_turn {
                self.advance_turn();
            }
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::common::boardposn::BoardPosn;

    #[test]
    fn test_new() {
        let board = Board::with_no_holes(3, 3, 3);
        let gamestate = GameState::new(board, 4); // create game with 4 players

        assert_eq!(gamestate.players.len(), 4);
        // should have 6-n penguins per player
        assert!(gamestate.players.iter().all(|(_, player)| player.penguins.len() == 2));

        // does turn_order contain each of the players' ids exactly once?
        assert_eq!(gamestate.turn_order.len(), gamestate.players.len());
        assert!(gamestate.players.iter().all(|(id, _)| gamestate.turn_order.contains(id)), "{:?},\nturns={:?}", gamestate.players, gamestate.turn_order);
        assert!(gamestate.winning_players.is_none()); // no winners yet
    }

    #[test]
    fn test_can_any_player_move_penguin() {
        // Can no players move when there's a penguin on the board, but holes blocking it in all directions?
        let holes = util::map_slice(&[(1, 1), (1, 0), (0, 1)], |pos| BoardPosn::from(*pos));
        let board_with_holes = Board::with_holes(2, 2, holes, 1);
        let mut gamestate = GameState::new(board_with_holes, 4);
        let player_id = *gamestate.players.iter().nth(0).unwrap().0;

        assert!(!gamestate.can_any_player_move_penguin());
        gamestate.place_avatar_without_changing_turn(player_id, TileId(0));
        assert!(!gamestate.can_any_player_move_penguin());


        // Can a player move when they have a penguin on the board with no holes blocking it?
        let board = Board::with_no_holes(3, 3, 3);
        let mut gamestate = GameState::new(board, 4);
        let player_id = *gamestate.players.iter().nth(0).unwrap().0;

        assert!(!gamestate.can_any_player_move_penguin());
        gamestate.place_avatar_without_changing_turn(player_id, TileId(0));
        assert!(gamestate.can_any_player_move_penguin());

        // Can no players move when all penguins are blocked by holes or other penguins?
        // 0(hole)      2(penguin)
        //    1(penguin)       3(hole)
        let holes = util::map_slice(&[(1, 1), (0, 0)], |pos| BoardPosn::from(*pos));
        let board_with_holes = Board::with_holes(2, 2, holes, 1);
        let mut gamestate = GameState::new(board_with_holes, 4);
        let player_id = *gamestate.players.iter().nth(0).unwrap().0;
        assert!(!gamestate.can_any_player_move_penguin());
        gamestate.place_avatar_without_changing_turn(player_id, TileId(1));
        assert!(&gamestate.can_any_player_move_penguin()); // no penguin at 2, so can move
        gamestate.place_avatar_without_changing_turn(player_id, TileId(2));
        assert!(!gamestate.can_any_player_move_penguin()); // penguin at 2, so cannot move
    }

    #[test]
    fn test_place_avatar() {
        let mut gamestate = GameState::with_default_board(3, 3, 2);
        gamestate.board.remove_tile(TileId(5));

        let player_id = *gamestate.players.iter().nth(0).unwrap().0;

        // Player places a penguin at a valid spot
        assert_eq!(gamestate.place_avatar_without_changing_turn(player_id, TileId(4)), Some(()));

        // Player tried to place a penguin at an invalid location
        assert_eq!(gamestate.place_avatar_without_changing_turn(player_id, TileId(10)), None);

        // Player tried to place a penguin at a hole
        assert_eq!(gamestate.place_avatar_without_changing_turn(player_id, TileId(5)), None);
    }

    #[test]
    fn test_move_avatar() {
        let mut gamestate = GameState::with_default_board(3, 3, 2);

        let player_id = *gamestate.players.iter().nth(0).unwrap().0;

        // Reachable tiles from 0 are [0, 2, 1, 5]
        let tile_0 = TileId(0);
        let reachable_tile = TileId(5);
        let unreachable_tile = TileId(3);

        // Move failed: penguin not yet placed
        assert_eq!(gamestate.move_avatar_for_player_without_changing_turn(player_id, tile_0, reachable_tile), None);

        gamestate.place_avatar_without_changing_turn(player_id, tile_0);

        // Move failed: tile not reachable from tile 0
        assert_eq!(gamestate.move_avatar_for_player_without_changing_turn(player_id, tile_0, tile_0), None);
        assert_eq!(gamestate.move_avatar_for_player_without_changing_turn(player_id, tile_0, unreachable_tile), None);

        // success, penguin should now be on tile 5
        assert_eq!(gamestate.move_avatar_for_player_without_changing_turn(player_id, tile_0, reachable_tile), Some(()));

        // Finally, assert that the position of the penguin actually changed
        let player = gamestate.players.iter_mut().nth(0).unwrap().1;
        let penguin_pos = player.find_penguin_mut(reachable_tile).and_then(|penguin| penguin.tile_id);
        assert_eq!(penguin_pos, Some(reachable_tile));
    }

    #[test]
    fn test_advance_turn() {
        let mut gamestate = GameState::with_default_board(3, 3, 4);

        for i in 0..4 {
            assert_eq!(gamestate.current_turn, gamestate.turn_order[i]);
            gamestate.advance_turn();
        }

        // check that advancing the turn on the last player makes the gamestate look at the first player in the order again
        assert_eq!(gamestate.current_turn, gamestate.turn_order[0]);
    }
}
