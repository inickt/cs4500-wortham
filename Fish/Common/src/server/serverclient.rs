use crate::server::connection::PlayerConnection;
use crate::client::player::InHousePlayer;
use crate::common::action::Action;

use std::io::Write;

use serde::Deserialize;

/// The server representation of a game client. There are 1 of
/// these per-player in a game and they are used to receive or
/// send messages between the server and that player through json.
pub enum Client {
    Remote(PlayerConnection),
    InHouseAI(InHousePlayer),
    Kicked,
}

impl Client {
    /// Get an action of the given player, either waiting for a remote player
    /// or prompting an ai player to take a turn.
    /// 
    /// TODO: Add 1 minute timeout for remote players
    pub fn get_action(&mut self) -> Option<Action> {
        match self {
            Client::Remote(connection) => {
                // Wait for the player to send their Action
                unimplemented!()
            },
            Client::InHouseAI(ai) => {
                ai.take_turn();
                serde_json::from_str(&mut ai.output_stream).ok()
            },
            Client::Kicked => unreachable!("It should never be a kicked player's turn"),
        }
    }

    /// Send a message to the player's input stream.
    /// 
    /// Since the possible server message to a player is that containing
    /// the current gamestate, it is expected the contents of this message
    /// contains the serialized gamestate.
    /// 
    /// Returns Ok(num_bytes_written) or otherwise returns an io error if
    /// the stream could not be written to.
    pub fn send(&mut self, message: &[u8]) -> Result<usize, std::io::Error> {
        match self {
            Client::Remote(connection) => {
                connection.stream.write(message)
            },
            Client::InHouseAI(ai) => { 
                ai.receive_gamestate(message);
                Ok(message.len())
            },
            Client::Kicked => Ok(0),
        }
    }

    pub fn is_kicked(&self) -> bool {
        match self {
            Client::Kicked => true,
            _ => false,
        }
    }
}