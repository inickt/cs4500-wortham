use crate::client::player::PlayerInterface;
use crate::common::action::{Action, Placement, Move, PlayerMove};
use crate::common::util;
use crate::common::gamestate::GameState;
use crate::common::game_tree::GameTree;
use crate::server::message::{ ClientToServerMessage, ServerToClientMessage, serialize_gamestate, convert_to_json_actions };

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

    fn call(&mut self, message: ServerToClientMessage) -> Option<ClientToServerMessage> {
        self.stream.write(message.serialize().as_bytes()).ok()?;
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

    fn get_placement(&mut self, gamestate: &GameState) -> Option<Placement> {
        let json_gamestate = serialize_gamestate(gamestate);
        match self.call(ServerToClientMessage::Setup((json_gamestate,)))? {
            ClientToServerMessage::Position(placement) => Some(placement),
            _ => None
        }
    }

    fn get_move(&mut self, gamestate: &GameState, previous: &[PlayerMove]) -> Option<Move> {
        let json_gamestate = serialize_gamestate(gamestate);
        let json_moves = convert_to_json_actions(&previous);
        match self.call(ServerToClientMessage::TakeTurn(json_gamestate, json_moves))? {
            ClientToServerMessage::Action(move_) => Some(move_),
            _ => None
        }
    }
}
