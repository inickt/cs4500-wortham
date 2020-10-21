//! This file contains a Rust API providing all the necessary functionality
//! for building a client application that can play the Fish game.
//! This Rust API is built off of the spec described in Planning/player-protocol.md.

/// Note that each function sends a message to a TcpStream, meaning
/// that stream must be connected to a Fish game server. If any function
/// is passed a connection that has been closed, they will panic with an
/// appropriate error message.


/// Sends a player connection message, informing the game server through the TCP
/// connection that this connection will be a player in this game.
/// Returns either the PlayerId assigned to the current player by the server if successful.
/// Returns Err when the game is full or has already started.
pub fn send_player_connect_message(stream: &mut TcpStream, age: u8) -> Result<PlayerId, String> { ... }

/// Send a PlacePenguin message to the game server via the given TCP stream
/// telling it to place one of this player's penguins. The game server will
/// determine which player sent the message based off their TCP connection info.
/// If it is not currently that player's turn, or the placement is otherwise
/// invalid, then they will be kicked from the game and their penguins will
/// be removed from the board.
pub fn send_place_penguin_message(stream: &mut TcpStream, destination_tile: TileId) { ... }

/// Send a MovePenguin message over tcp to tell the server to move a given
/// penguin to the given destination tile. If the penguin has not yet been
/// placed, does not belong to the current player, or the move itself is
/// oherwise invalid, the player will be kicked from the game.
/// 
/// The game server will determine
/// which player sent the message based off their TCP connection info.
/// If it is not currently that player's turn then they will be kicked
/// from the game and their penguins will be removed from the board.
pub fn send_move_penguin_message(stream: &mut TcpStream, penguin_id: PenguinId, destination_tile: TileId) { ... }

/// Block until the server sends a game state at the start of the next turn,
/// then returns the GameState once one is received.
/// 
/// A state will get sent from the server any time an action is performed
/// by a player that changes the game state (placing a penguin, moving a penguin).
/// This state is automatically sent to every player and it is the player's job
/// to recieve the gamestate via receive_gamestate()
/// 
/// This function returns a GameState, which contains all the information
/// about the given game. Players may wish to view `src/common/gamestate.rs`
/// for more information and documentation about how to work with this
/// struct. Players may also wish to use Game struct to check if their planned
/// moves are valid, or to peek ahead into the future.
pub fn receive_gamestate(stream: &mut TcpStream) -> GameState { ... }