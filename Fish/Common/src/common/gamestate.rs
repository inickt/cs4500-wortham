//! The gamestate module defines the SharedGameState that will be
//! serialized by the server and sent to each client - informing
//! them of any updates to the game. The GameState itself is a
//! shared mutable pointer which in the client is shared between
//! the communication layer (TBD) and the ui layer.
use crate::common::board::Board;
use crate::common::boardposn::BoardPosn;
use crate::common::tile::{ TileId, Tile };
use crate::common::player::{ Player, PlayerId, PlayerColor };
use crate::common::penguin::{ Penguin, PenguinId };
use crate::common::util;

use std::collections::HashSet;
use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;

const MIN_PLAYERS_PER_GAME: usize = 2;
const MAX_PLAYERS_PER_GAME: usize = 4;

#[derive(Debug)]
pub struct GameId(usize);

// Rc<RefCell<T>> gives a copiable, mutable reference to its T
pub type SharedGameState = Rc<RefCell<GameState>>;

/// The SharedGameState contains the entirety of the current state
/// of the game. It is meant to be serialized into json from the server
/// and sent to each client to deserialize to receive the updated game
/// state each turn. The SharedGameState is rendering-agnostic, so each
/// client is free to render the SharedGameState however it wishes.
#[derive(Debug)]
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

        let players: HashMap<_, _> = util::make_n(player_count, |_| {
            let penguins = util::make_n(penguins_per_player, |_| Penguin::new());
            let player = Player::new(penguins);
            (player.player_id, player)
        });

        let turn_order = players.keys().copied().collect(); // TODO sort by age 

        Rc::new(RefCell::new(GameState {
            game_id: GameId(id),
            board,
            players,
            turn_order,
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
        let occupied = &self.get_occupied_tiles();
        let player = self.players.get_mut(&player)?;
        player.move_penguin(penguin, destination, &self.board, occupied)
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
        self.players.iter()
            .find_map(|(_, player)| {
                let is_penguin_on_tile = player.penguins.iter().any(|penguin| penguin.tile_id == Some(tile_id));
                if is_penguin_on_tile {
                    Some(player.color)
                } else {
                    None
                }
            })
    }

    /// Returns true if any player has a penguin they can move,
    /// false if not (the game is over)
    pub fn can_any_player_move_penguin(&self) -> bool {
        self.players.iter().any(|(_, player)| dbg!(player.can_move_a_penguin(&self.board, &self.get_occupied_tiles())))
    }

    /// Returns the set of tiles on this gamestate's board which have a penguin on them
    pub fn get_occupied_tiles(&self) -> HashSet<TileId> {
        self.players.iter()
            .flat_map(|(_, player)| player.penguins.iter().filter_map(|penguin| penguin.tile_id))
            .collect()
    }
}

#[test]
fn test_new() {
    let board = Board::with_no_holes(3, 3, 3);
    let gamestate = GameState::new(1, board, 4); // create game with 4 players
    let gamestate = gamestate.borrow();

    assert_eq!(gamestate.players.len(), 4);
    // should have 6-n penguins per player
    assert!(gamestate.players.iter().all(|(_, player)| player.penguins.len() == 2));

    // does turn_order contain each of the players' ids exactly once?
    assert_eq!(gamestate.turn_order.len(), gamestate.players.len());
    assert!(gamestate.players.iter().all(|(id, _)| gamestate.turn_order.contains(id)), "{:?},\nturns={:?}", gamestate.players, gamestate.turn_order);
    assert_eq!(gamestate.winning_players, None); // no winners yet
}

#[test]
fn test_can_any_player_move_penguin() {
    // Can no players move when there's a penguin on the board, but holes blocking it in all directions?
    let holes = util::map_slice(&[(1, 1), (1, 0), (0, 1)], |pos| BoardPosn::from(*pos));
    let board_with_holes = Board::with_holes(2, 2, holes, 1);
    let gamestate_unmovable = GameState::new(1, board_with_holes, 4);
    let mut gamestate = gamestate_unmovable.borrow_mut();
    let (&player_id, player) = gamestate.players.iter().nth(0).unwrap();
    let penguin_id = player.penguins[0].penguin_id;
    assert!(!gamestate.can_any_player_move_penguin());
    gamestate.place_avatar_for_player(player_id, penguin_id, TileId(0));
    assert!(!gamestate.can_any_player_move_penguin());


    // Can a player move when they have a penguin on the board with no holes blocking it?
    let board = Board::with_no_holes(3, 3, 3);
    let gamestate = GameState::new(1, board, 4);
    let mut gamestate = gamestate.borrow_mut();
    let (&player_id, player) = gamestate.players.iter().nth(0).unwrap();
    let penguin_id = player.penguins[0].penguin_id;
    assert!(!gamestate.can_any_player_move_penguin());
    gamestate.place_avatar_for_player(player_id, penguin_id, TileId(0));
    assert!(gamestate.can_any_player_move_penguin());

    // Can no players move when all penguins are blocked by holes or other penguins?
    // 0(hole)      2(penguin)
    //    1(penguin)       3(hole)
    let holes = util::map_slice(&[(1, 1), (0, 0)], |pos| BoardPosn::from(*pos));
    let board_with_holes = Board::with_holes(2, 2, holes, 1);
    let gamestate_unmovable = GameState::new(1, board_with_holes, 4);
    let mut gamestate = gamestate_unmovable.borrow_mut();
    let (&player_id, player) = gamestate.players.iter().nth(0).unwrap();
    let penguin_id = player.penguins[0].penguin_id;
    let penguin_id_2 = player.penguins[1].penguin_id;
    assert!(!gamestate.can_any_player_move_penguin());
    gamestate.place_avatar_for_player(player_id, penguin_id, TileId(1));
    assert!(&gamestate.can_any_player_move_penguin()); // no penguin at 2, so can move
    gamestate.place_avatar_for_player(player_id, penguin_id_2, TileId(2));
    assert!(!gamestate.can_any_player_move_penguin()); // penguin at 2, so cannot move
}