use crate::common::action::{ Placement, Move, PlayerMove };
use crate::common::gamestate::GameState;
use crate::common::player::PlayerColor;
use crate::common::util;
use crate::server::client::Client;
use crate::server::message::*;

use std::net::TcpStream;
use std::time::Duration;
use std::io::Write;

use serde::Deserialize;
use serde_json::Deserializer;

/// A remote client that is communicated with only through TcpStream.
/// This RemoteClient will handle serialization of each ServerToClientMessage
/// into json and sending them through tcp.
///
/// See ClientToServerProxy for the other side of this connection which will
/// handle deserialization of each message and running the client-side main loop
/// until the game is over.
pub struct RemoteClient {
    stream: TcpStream,
    timeout: Duration,
}

impl RemoteClient {
    /// Creates a new RemoteClient from the given stream, setting both
    /// read and write timeouts to the given Duration.
    pub fn new(stream: TcpStream, timeout: Duration) -> RemoteClient {
        stream.set_read_timeout(Some(timeout)).unwrap();
        stream.set_write_timeout(Some(timeout)).unwrap();
        RemoteClient { stream, timeout }
    }

    /// Receives and validates a name from the given TcpStream.
    /// A valid name:
    /// - Is between 1 and 12 characters inclusive
    /// - Consists of only ascii alphabetic characters
    pub fn get_name(&mut self, timeout: Duration) -> Option<String> {
        let name: String = self.receive_with_timeout(timeout)?;
        if !name.is_empty() && name.len() <= 12 && name.chars().all(|c| c.is_ascii_alphabetic()) {
            Some(name)
        } else {
            None
        }
    }

    fn receive<'a, T: Deserialize<'a>>(&mut self) -> Option<T> {
        self.receive_with_timeout(self.timeout)
    }

    fn receive_with_timeout<'a, T: Deserialize<'a>>(&mut self, timeout: Duration) -> Option<T> {
        let mut de = Deserializer::from_reader(self.stream.try_clone().unwrap());
        util::try_with_timeout(timeout, || {
            T::deserialize(&mut de).ok()
        })
    }

    fn call(&mut self, message: String) -> Option<ClientToServerMessage> {
        self.stream.write(message.as_bytes()).ok()?;
        self.receive()
    }

    fn void_call(&mut self, message: String) -> Option<()> {
        match self.call(message)? {
            ClientToServerMessage::Void(_) => Some(()),
            _ => None
        }
    }
}

impl Client for RemoteClient {
    fn tournament_starting(&mut self) -> Option<()> {
        self.void_call(start_message())
    }

    fn tournament_ending(&mut self, won: bool) -> Option<()> {
        self.void_call(end_message(won))
    }

    fn initialize_game(&mut self, initial_gamestate: &GameState, player_color: PlayerColor) -> Option<()> {
        self.void_call(playing_as_message(player_color))?;

        let other_colors = initial_gamestate.players.iter()
            .map(|player| player.1.color)
            .filter(|color| *color != player_color)
            .collect::<Vec<PlayerColor>>();

        self.void_call(playing_with_message(&other_colors))
    }

    fn get_placement(&mut self, gamestate: &GameState) -> Option<Placement> {
        match self.call(setup_message(gamestate))? {
            ClientToServerMessage::Position(json_placement) => {
                let tile_id = gamestate.board.get_tile_id(json_placement[1], json_placement[0])?;
                Some(Placement::new(tile_id))
            },
            _ => None
        }
    }

    fn get_move(&mut self, gamestate: &GameState, previous: &[PlayerMove]) -> Option<Move> {
        match self.call(take_turn_message(gamestate, previous))? {
            ClientToServerMessage::Action(json_move) => {
                let from_tile_id = gamestate.board.get_tile_id(json_move[0][1], json_move[0][0])?;
                let to_tile_id = gamestate.board.get_tile_id(json_move[1][1], json_move[1][0])?;
                Some(Move::new(from_tile_id, to_tile_id))
            },
            _ => None
        }
    }
}
