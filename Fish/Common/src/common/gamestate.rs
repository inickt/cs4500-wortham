//! The gamestate module defines the SharedGameState that will be
//! serialized by the server and sent to each client - informing
//! them of any updates to the game. The GameState itself is a
//! shared mutable pointer which in the client is shared between
//! the communication layer (TBD) and the ui layer. It represents
//! the full state of the game at any given point in time.
use crate::common::board::Board;
use crate::common::tile::{ TileId, Tile };
use crate::common::player::{ Player, PlayerId, PlayerColor };
use crate::common::penguin::{Penguin, PenguinId};
use crate::common::action::Move;
use crate::common::boardposn::BoardPosn;
use crate::common::util;

use std::collections::HashSet;
use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;

const MIN_PLAYERS_PER_GAME: usize = 2;
const MAX_PLAYERS_PER_GAME: usize = 4;

/// Each player receives 6 - player_count penguins to start the game
const PENGUIN_FACTOR: usize = 6;

#[derive(Copy, Clone, Debug)]
pub struct GameId(usize);

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
#[derive(Clone, Debug)]
pub struct GameState {
    pub game_id: GameId,
    pub board: Board,
    pub players: HashMap<PlayerId, Player>,
    pub turn_order: Vec<PlayerId>, // INVARIANT: turn_order never changes for a given game
    pub current_turn: PlayerId,
    pub spectator_count: usize, // simple count so that players can see their audience size
    pub winning_players: Vec<PlayerId>, // will be empty until the game ends
}

impl GameState {
    /// Create a new SharedGameState with the given game id, board, and player_count.
    /// This will panic if player_count is < MIN_PLAYERS_PER_GAME or > MAX_PLAYERS_PER_GAME.
    pub fn new(game_id: usize, board: Board, player_count: usize) -> GameState {
        assert!(player_count >= MIN_PLAYERS_PER_GAME, "Fish must be played with at least {} players!", MIN_PLAYERS_PER_GAME);
        assert!(player_count <= MAX_PLAYERS_PER_GAME, "Fish only supports up to {} players!", MAX_PLAYERS_PER_GAME);

        // Each player receives 6 - N penguins, where N is the number of players
        let penguins_per_player = PENGUIN_FACTOR - player_count; 

        let players: HashMap<_, _> = util::make_n(player_count, |_| {
            let player = Player::new(penguins_per_player);
            (player.player_id, player)
        });

        let turn_order: Vec<PlayerId> = players.keys().copied().collect(); // TODO sort by age 
        let current_turn = turn_order[0];

        GameState {
            game_id: GameId(game_id),
            board,
            players,
            turn_order,
            current_turn,
            spectator_count: 0,
            winning_players: vec![],
        }
    }

    pub fn with_default_board(rows: u32, columns: u32, players: usize) -> GameState {
        let board = Board::with_no_holes(rows, columns, 3);
        GameState::new(1, board, players)
    }

    /// Places an unplaced avatar on a position on the board, and advances the turn. 
    /// Returns Some(()) on success, or None if the player makes an invalid placement.
    /// An invalid placement is one of:
    /// 1. Placement on an invalid position (either out of bounds or a hole)
    /// 2. Placement when the players' avatars are already placed
    /// 3. Placement of a penguin that doesn't belong to the current player
    pub fn place_avatar_for_player(&mut self, player: PlayerId, penguin: PenguinId, tile: TileId) -> Option<()> {
        self.place_avatar_without_changing_turn(player, penguin, tile)?;
        self.advance_turn();
        Some(())
    }

    /// Place a player's avatar but don't change whose turn it is.
    /// This is useful to more easily place avatars in bulk during testing.
    pub fn place_avatar_without_changing_turn(&mut self, player: PlayerId, penguin: PenguinId, tile: TileId) -> Option<()> {
        let player = self.players.get_mut(&player)?; 
        player.place_penguin(penguin, tile, &self.board)
    }

    /// Moves a placed avatar from one position to another on the board,
    /// removes the tile that penguin was on, and advances the turn.
    /// Returns Some(()) on success, or None if the player makes an invalid move.
    /// An invalid placement is one of:
    /// 1. Move to an invalid position (either out of bounds or hole)
    /// 2. Move when the player has other unplaced avatars
    /// 3. Move when the current avatar has yet to be placed
    /// 4. Placement on a tile that is not accessible within a straight line
    ///    of the current tile, with no holes in between.
    /// 5. Move a penguin that doesn't belong to the player
    pub fn move_avatar_for_player_without_changing_turn(&mut self, player: PlayerId, penguin: PenguinId, destination: TileId) -> Option<()> {
        let occupied = &self.get_occupied_tiles();
        let player = self.players.get_mut(&player)?;
        let current_tile = player.find_penguin(penguin)?.tile_id?;
        player.move_penguin(penguin, destination, &self.board, occupied)?;
        player.score += self.board.remove_tile(current_tile);
        Some(())
    }

    /// Helper function which moves an avatar for the player whose turn it currently is.
    pub fn move_avatar_for_current_player(&mut self, move_: Move) -> Option<()> {
        self.move_avatar_for_player_without_changing_turn(self.current_turn, move_.penguin_id, move_.tile_id)?;
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
        self.players.iter().any(|(_, player)| player.can_move_a_penguin(&self.board, &self.get_occupied_tiles()))
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
        let penguins_to_move = &self.players[&self.current_turn].penguins;

        penguins_to_move.iter().flat_map(|penguin| {
            // penguins in Games are placed, so should always be Some
            let starting_tile_id = penguin.tile_id.expect(&format!("Penguin {:?} was not placed!", penguin.penguin_id)); 
            let starting_tile = self.get_tile(starting_tile_id).unwrap();
            let penguin_id = penguin.penguin_id;

            starting_tile.all_reachable_tiles(&self.board, &occupied_tiles)
                .into_iter()
                .map(move |destination| Move::new(penguin_id, destination.tile_id))
        }).collect()
    }

    // Collect penguins for every player, along with which player they belong to
    pub fn all_penguins(&self) -> Vec<(PlayerId, PenguinId)> {
        self.players.iter().flat_map(|(&player_id, player)| {
            player.penguins.iter().map(move |penguin| (player_id, penguin.penguin_id))
        }).collect()
    }

    /// Get a penguin given its PenguinId
    pub fn find_penguin(&self, id: PenguinId) -> Option<&Penguin> {
        self.players.iter().find_map(|(_, player)| player.find_penguin(id))
    }

    /// Get a penguin at a position, None if no penguin at that position
    pub fn find_penguin_at_position(&self, posn: BoardPosn) -> Option<&Penguin> {
        let tile = self.board.get_tile(posn.x, posn.y)?;
        self.players.iter().find_map(|(&player_id, player)| {
            player.penguins.iter().find(|penguin| {
                match penguin.tile_id {
                    Some(tile_id) => tile_id == tile.tile_id,
                    None => false
                }
            })
        })
    }

    pub fn get_penguin_tile_position(&self, penguin_id: PenguinId) -> Option<BoardPosn> {
        let penguin = self.find_penguin(penguin_id)?;
        penguin.tile_id.map(|id| self.board.get_tile_position(id))
    }

    /// Returns the player whose turn it currently is
    pub fn current_player(&self) -> &Player {
        self.players.get(&self.current_turn).unwrap()
    }

    pub fn is_game_over(&self) -> bool {
        let game_over = !self.winning_players.is_empty();
        assert_ne!(self.can_any_player_move_penguin(), game_over);
        game_over
    }

    /// Advance the turn of this game to the next player's turn
    /// Will mutate this game's current_turn field
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
        self.winning_players = util::all_max_by_key(self.players.iter(), |(_, player)| player.score)
            .map(|(id, _)| *id).collect();
    }

    /// Sets the turn of this game to the next player in order
    fn advance_turn_index(&mut self) {
        let current_turn_index = self.turn_order.iter().position(|id| id == &self.current_turn).unwrap();
        let next_turn_index = (current_turn_index + 1) % self.turn_order.len();
        self.current_turn = self.turn_order[next_turn_index];
    }

    pub fn player_score(&self, player_id: PlayerId) -> usize {
        self.players[&player_id].score
    }

    /// Returns true if the two tiles are adjacent or false if either tile is a hole.
    pub fn tiles_are_adjacent(&self, tile1: TileId, tile2: TileId) -> bool {
        match (self.get_tile(tile1), self.get_tile(tile2)) {
            (Some(tile1), Some(tile2)) => {
                tile1.north == Some(tile2.tile_id)
                || tile1.northeast == Some(tile2.tile_id)
                || tile1.southeast == Some(tile2.tile_id)
                || tile1.south == Some(tile2.tile_id)
                || tile1.southwest == Some(tile2.tile_id)
                || tile1.northwest == Some(tile2.tile_id)
            },
            _ => false,
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
        let gamestate = GameState::new(1, board, 4); // create game with 4 players

        assert_eq!(gamestate.players.len(), 4);
        // should have 6-n penguins per player
        assert!(gamestate.players.iter().all(|(_, player)| player.penguins.len() == 2));

        // does turn_order contain each of the players' ids exactly once?
        assert_eq!(gamestate.turn_order.len(), gamestate.players.len());
        assert!(gamestate.players.iter().all(|(id, _)| gamestate.turn_order.contains(id)), "{:?},\nturns={:?}", gamestate.players, gamestate.turn_order);
        assert!(gamestate.winning_players.is_empty()); // no winners yet
    }

    #[test]
    fn test_can_any_player_move_penguin() {
        // Can no players move when there's a penguin on the board, but holes blocking it in all directions?
        let holes = util::map_slice(&[(1, 1), (1, 0), (0, 1)], |pos| BoardPosn::from(*pos));
        let board_with_holes = Board::with_holes(2, 2, holes, 1);
        let mut gamestate = GameState::new(1, board_with_holes, 4);
        let (&player_id, player) = gamestate.players.iter().nth(0).unwrap();
        let penguin_id = player.penguins[0].penguin_id;
        assert!(!gamestate.can_any_player_move_penguin());
        gamestate.place_avatar_without_changing_turn(player_id, penguin_id, TileId(0));
        assert!(!gamestate.can_any_player_move_penguin());


        // Can a player move when they have a penguin on the board with no holes blocking it?
        let board = Board::with_no_holes(3, 3, 3);
        let mut gamestate = GameState::new(1, board, 4);
        let (&player_id, player) = gamestate.players.iter().nth(0).unwrap();
        let penguin_id = player.penguins[0].penguin_id;
        assert!(!gamestate.can_any_player_move_penguin());
        gamestate.place_avatar_without_changing_turn(player_id, penguin_id, TileId(0));
        assert!(gamestate.can_any_player_move_penguin());

        // Can no players move when all penguins are blocked by holes or other penguins?
        // 0(hole)      2(penguin)
        //    1(penguin)       3(hole)
        let holes = util::map_slice(&[(1, 1), (0, 0)], |pos| BoardPosn::from(*pos));
        let board_with_holes = Board::with_holes(2, 2, holes, 1);
        let mut gamestate = GameState::new(1, board_with_holes, 4);
        let (&player_id, player) = gamestate.players.iter().nth(0).unwrap();
        let penguin_id = player.penguins[0].penguin_id;
        let penguin_id_2 = player.penguins[1].penguin_id;
        assert!(!gamestate.can_any_player_move_penguin());
        gamestate.place_avatar_without_changing_turn(player_id, penguin_id, TileId(1));
        assert!(&gamestate.can_any_player_move_penguin()); // no penguin at 2, so can move
        gamestate.place_avatar_without_changing_turn(player_id, penguin_id_2, TileId(2));
        assert!(!gamestate.can_any_player_move_penguin()); // penguin at 2, so cannot move
    }

    #[test]
    fn test_place_avatar() {
        let mut gamestate = GameState::with_default_board(3, 3, 2);
        gamestate.board.remove_tile(TileId(5));

        let (&player_id, player) = gamestate.players.iter().nth(0).unwrap();

        let unowned_penguin = crate::common::penguin::Penguin::new();
        let penguin1 = player.penguins[0].penguin_id;
        let penguin2 = player.penguins[1].penguin_id;

        // Player tried to place down a penguin they don't own
        assert_eq!(gamestate.place_avatar_without_changing_turn(player_id, unowned_penguin.penguin_id, TileId(4)), None);

        // Player places a penguin at a valid spot
        assert_eq!(gamestate.place_avatar_without_changing_turn(player_id, penguin1, TileId(4)), Some(()));

        // Placing an already-placed penguin is invalid
        assert_eq!(gamestate.place_avatar_without_changing_turn(player_id, penguin1, TileId(4)), None);

        // Player tried to place a penguin at an invalid location
        assert_eq!(gamestate.place_avatar_without_changing_turn(player_id, penguin2, TileId(10)), None);

        // Player tried to place a penguin at a hole
        assert_eq!(gamestate.place_avatar_without_changing_turn(player_id, penguin2, TileId(5)), None);
    }

    #[test]
    fn test_move_avatar() {
        let mut gamestate = GameState::with_default_board(3, 3, 2);

        let (&player_id, player) = gamestate.players.iter().nth(0).unwrap();
        let penguin_id = player.penguins[0].penguin_id;

        // Reachable tiles from 0 are [0, 2, 1, 5]
        let tile_0 = TileId(0);
        let reachable_tile = TileId(5);
        let unreachable_tile = TileId(3);

        // Move failed: penguin not yet placed
        assert_eq!(gamestate.move_avatar_for_player_without_changing_turn(player_id, penguin_id, tile_0), None);

        gamestate.place_avatar_without_changing_turn(player_id, penguin_id, tile_0);

        // Move failed: tile not reachable from tile 0
        assert_eq!(gamestate.move_avatar_for_player_without_changing_turn(player_id, penguin_id, tile_0), None);
        assert_eq!(gamestate.move_avatar_for_player_without_changing_turn(player_id, penguin_id, unreachable_tile), None);

        // success, penguin should now be on tile 5
        assert_eq!(gamestate.move_avatar_for_player_without_changing_turn(player_id, penguin_id, reachable_tile), Some(()));

        // Finally, assert that the position of the penguin actually changed
        let player = gamestate.players.iter_mut().nth(0).unwrap().1;
        let penguin_pos = player.find_penguin_mut(penguin_id).and_then(|penguin| penguin.tile_id);
        assert_eq!(penguin_pos, Some(reachable_tile));
    }
}
