//! Each submodule within the common module contains code
//! common to the client and server. This is mostly model code dealing
//! with public information about the Board, its Tiles, and each Player.
//!
//! The Board and its contained Tiles are part of the GameState, and are thus
//! serialized to be sent over the network from the server to each client (TBD).
pub mod board;
pub mod boardposn;
pub mod gamestate;
pub mod player;
pub mod tile;

mod direction;
mod penguin;
mod util;