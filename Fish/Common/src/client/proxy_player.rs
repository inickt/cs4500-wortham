use crate::client::player::PlayerInterface;
use crate::common::action::{Action, Placement, Move, PlayerMove};
use crate::common::util;
use crate::common::gamestate::GameState;
use crate::common::game_tree::GameTree;
use crate::server::message::{ ClientToServerMessage, ServerToClientMessage };

use std::net::TcpStream;
use std::time::Duration;
use std::io::{ Error, Write };

use serde::Deserialize;
use serde_json::Deserializer;

pub struct ProxyPlayer {
    stream: TcpStream,
    timeout: Duration
}

impl ProxyPlayer {
    pub fn new(stream: TcpStream, timeout: Duration) -> ProxyPlayer {
        stream.set_read_timeout(Some(timeout)).unwrap();
        stream.set_write_timeout(Some(timeout)).unwrap();
        ProxyPlayer { stream, timeout }
    }

    fn receive<'a, T: Deserialize<'a>>(&mut self) -> Option<T> {
        let mut de = Deserializer::from_reader(self.stream.try_clone().unwrap());
        util::try_with_timeout(self.timeout, || {
            T::deserialize(&mut de).ok()
        })
    }

    // TODO probably don't need this abstraction since we need to convert to the JSON 
    // representations before this anyways
    fn send(&mut self, message: ServerToClientMessage) -> Option<()> {
        // TODO can we one line this?   
        self.stream.write(message.serialize().as_bytes()).ok()?;
        Some(())
    }

    fn call(&mut self, message: ServerToClientMessage) -> Option<ClientToServerMessage> {
        self.send(message)?;
        self.receive()
    }

}

impl PlayerInterface for ProxyPlayer {
    fn tournament_starting(&mut self) -> Option<()> {
        match self.call(ServerToClientMessage::Start((true,)))? {
            ClientToServerMessage::Void() => Some(()),
            _ => None
        }
    }

    fn tournament_ending(&mut self, won: bool) -> Option<()> {
        match self.call(ServerToClientMessage::End((won,)))? {
            ClientToServerMessage::Void() => Some(()),
            _ => None
        }
    }

    // TODO need to add starting color to this and any other info
    fn initialize_game(&mut self, initial_gamestate: &GameState) -> Option<()> {
        match self.call(ServerToClientMessage::PlayingAs(unimplemented!()))? {
            ClientToServerMessage::Void() => Some(()),
            _ => None
        }?;
        match self.call(ServerToClientMessage::PlayingWith((unimplemented!())))? {
            ClientToServerMessage::Void() => Some(()),
            _ => None
        }
    }

    fn get_placement(&mut self, gamestate: &GameState, previous: Vec<PlayerMove>) -> Option<Placement> {
        unimplemented!()
    }

    fn get_move(&mut self, game: &mut GameState) -> Option<Move> {
        unimplemented!()
    }
}
