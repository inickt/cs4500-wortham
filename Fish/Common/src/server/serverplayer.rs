use crate::server::connection::PlayerConnection;
use crate::client::player::InHousePlayer;
use crate::common::action::Action;

use serde::Deserialize;

pub enum Client {
    Remote(PlayerConnection),
    InHouseAI(InHousePlayer),
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
                Action::deserialize(&mut connection.input_deserializer).ok()
            },
            Client::InHouseAI(ai) => {
                ai.take_turn();
                serde_json::from_str(&mut ai.output_stream).ok()
            }
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
                connection.output_stream.write(message)
            },
            Client::InHouseAI(ai) => { 
                ai.receive_gamestate(message);
                Ok(message.len())
            },
        }
    }
}
