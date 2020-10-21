//! Each submodule within the common module contains code
//! common to the client and server. This is mostly model code dealing
//! with public information about the Board, its Tiles, and each Player.
//!
//! The Board and its contained Tiles are part of the GameState, and are thus
//! serialized to be sent over the network from the server to each client (TBD).
pub mod action;
pub mod board;
pub mod boardposn;
pub mod direction;
pub mod gamestate;
pub mod penguin;
pub mod player;
pub mod tile;
pub mod util;
pub mod game_tree;
