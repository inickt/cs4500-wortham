//! The gamestate module defines the SharedGameState that will be
//! serialized by the server and sent to each client - informing
//! them of any updates to the game. The GameState itself is a
//! shared mutable pointer which in the client is shared between
//! the communication layer (TBD) and the ui layer.
use crate::common::board::Board;
use crate::common::boardposn::BoardPosn;
use crate::common::tile::{ TileId, Tile };
use crate::common::player::{ Player, PlayerId };
use crate::common::penguin::{ Penguin, PenguinId };
use crate::common::util;

use std::clone::Clone;
use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;

const MIN_PLAYERS_PER_GAME: usize = 2;
const MAX_PLAYERS_PER_GAME: usize = 4;

pub struct GameId(usize);

// Rc<RefCell<T>> gives a copiable, mutable reference to its T
pub type SharedGameState = Rc<RefCell<GameState>>;

/// The SharedGameState contains the entirety of the current state
/// of the game. It is meant to be serialized into json from the server
/// and sent to each client to deserialize to receive the updated game
/// state each turn. The SharedGameState is rendering-agnostic, so each
/// client is free to render the SharedGameState however it wishes.
pub struct GameState {
    pub game_id: GameId,
    pub board: Board,
    pub players: HashMap<PlayerId, Player>,
    pub turn_order: Vec<PlayerId>,
    pub current_turn: PlayerId,
    pub spectator_count: usize,
    pub winning_players: Option<Vec<PlayerId>>,
}

impl GameState {
    /// Convenience function for creating a new gamestate containing a
    /// board with the given specifications.
    pub fn new(id: usize, board: Board, player_count: usize) -> SharedGameState {
        assert!(player_count >= MIN_PLAYERS_PER_GAME, "Fish must be played with at least {} players!", MIN_PLAYERS_PER_GAME);
        assert!(player_count <= MAX_PLAYERS_PER_GAME, "Fish only supports up to {} players!", MAX_PLAYERS_PER_GAME);

        // Each player receives 6 - N penguins, where N is the number of players
        let penguins_per_player = 6 - player_count; 

        let players = util::make_n(player_count, |_| {
            let penguins = util::make_n(penguins_per_player, |_| Penguin::new());
            let player = Player::new(penguins);
            (player.player_id, player)
        });

        Rc::new(RefCell::new(GameState {
            game_id: GameId(id),
            board,
            players,
            turn_order: util::make_n(player_count, PlayerId),
            current_turn: PlayerId(0),
            spectator_count: 0,
            winning_players: None,
        }))
    }

    /// Places an unplaced avatar on a position on the board. 
    /// Returns true on success, false if the player makes an invalid placement.
    /// An invalid placement is one of:
    /// 1. Placement on an invalid position (either out of bounds or a hole)
    /// 2. Placement when the players' avatars are already placed
    /// 3. Placement of a penguin that doesn't belong to the current player
    pub fn place_avatar_for_player(&mut self, player: PlayerId, penguin: PenguinId, tile: TileId) -> Option<()> {
        let player = self.players.get_mut(&player)?; 
        player.place_penguin(penguin, tile, &self.board)
    }

    /// Moves a placed avatar from one position to another on the board. 
    /// Returns true on success, false if the player makes an invalid move.
    /// An invalid placement is one of:
    /// 1. Move to an invalid position (either out of bounds or hole)
    /// 2. Move when the player has other unplaced avatars
    /// 3. Move when the current avatar has yet to be placed
    /// 4. Placement on a tile that is not accessible within a straight line
    ///    of the current tile, with no holes in between.
    /// 5. Move a penguin that doesn't belong to the player
    pub fn move_avatar_for_player(&mut self, player: PlayerId, penguin: PenguinId, destination: TileId) -> Option<()> {
        let player = self.players.get_mut(&player)?;
        player.move_penguin(penguin, destination, &self.board)
    }

    /// Retrieve a tile by its ID. Will return None if the id
    /// does not reference any existing tile. This can happen
    /// if the tile was removed and has become a hole in the board.
    pub fn get_tile(&self, tile_id: TileId) -> Option<&Tile> {
        self.board.tiles.get(&tile_id)
    }
}