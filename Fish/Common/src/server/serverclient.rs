use crate::server::connection::PlayerConnection;
use crate::client::player::InHousePlayer;
use crate::common::action::Action;
use crate::common::player::PlayerId;

use std::cell::RefCell;
use std::rc::Rc;

/// Represents the client's connection info along with an
/// id to identify that particular client across all tournament games.
#[derive(Clone)]
pub struct Client {
    pub id: PlayerId,

    /// This is the shared, mutable reference to the ClientProxy shared
    /// between the tournament manager and the referee components.
    pub proxy: Rc<RefCell<ClientProxy>>,
}

impl Client {
    pub fn new(id: usize, proxy: ClientProxy) -> Client {
        Client {
            id: PlayerId(id),
            proxy: Rc::new(RefCell::new(proxy)),
        }
    }
}

/// The server representation of a game client. There are 1 of
/// these per-player in a game and they are used to receive or
/// send messages between the server and that player through json.
pub enum ClientProxy {
    Remote(PlayerConnection),
    InHouseAI(InHousePlayer),
    Kicked,
}

impl ClientProxy {
    /// Get an action of the given player, either waiting for a remote player
    /// or prompting an ai player to take a turn.
    pub fn get_action(&mut self) -> Option<Action> {
        match self {
            ClientProxy::Remote(connection) => {
                connection.receive_action()
            },
            ClientProxy::InHouseAI(ai) => {
                Some(ai.take_turn())
            },
            ClientProxy::Kicked => unreachable!("It should never be a kicked player's turn"),
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
            ClientProxy::Remote(connection) => {
                connection.write(message)
            },
            ClientProxy::InHouseAI(ai) => { 
                ai.receive_gamestate(message);
                Ok(message.len())
            },
            ClientProxy::Kicked => Ok(0),
        }
    }

    pub fn is_kicked(&self) -> bool {
        match self {
            ClientProxy::Kicked => true,
            _ => false,
        }
    }
}
