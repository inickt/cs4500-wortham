use crate::server::client::Client;
use crate::common::util;
use crate::server::message::*;

use std::net::TcpStream;
use std::time::Duration;
use std::io::{ Error, Write };

use serde::Deserialize;
use serde_json::Deserializer;

pub struct ClientToServerProxy {
    name: String,
    client: Box<dyn Client>,
    stream: TcpStream,
    timeout: Duration,
}

impl ClientToServerProxy {
    pub fn new(name: String, client: Box<dyn Client>, address: &str, timeout: Duration) -> Option<ClientToServerProxy> {
        let stream = TcpStream::connect(address).ok()?;
        stream.set_read_timeout(Some(timeout)).unwrap();
        stream.set_write_timeout(Some(timeout)).unwrap();
        Some(ClientToServerProxy { name, client, stream, timeout })
    }

    pub fn tournament_loop(&mut self) -> Option<bool> {
        self.send(ClientToServerMessage::Name(self.name))?;
        loop {
            match self.receive()? {
                ServerToClientMessage::Start(_) => {
                    self.client.tournament_starting()?;
                    self.send(ClientToServerMessage::Void(JSONVoid::Void))?;
                },
                ServerToClientMessage::End((won,)) => {
                    self.client.tournament_ending(won)?;
                    self.send(ClientToServerMessage::Void(JSONVoid::Void))?;
                    return Some(won)
                },
                ServerToClientMessage::PlayingAs((color,)) => {
                    // TODO shoot this is where this falls through the cracks. what to do about initialize?
                    self.send(ClientToServerMessage::Void(JSONVoid::Void))?;
                },
                ServerToClientMessage::PlayingWith((other_colors,)) => {
                    // TODO shoot this is where this falls through the cracks. what to do about initialize?
                    self.send(ClientToServerMessage::Void(JSONVoid::Void))?;
                },
                ServerToClientMessage::Setup((json_gamestate,)) => {
                    let placement = self.client.get_placement(&json_gamestate.to_common_game_state())?;
                    self.send(ClientToServerMessage::Position(placement_to_json_position(placement)))?;
                },
                ServerToClientMessage::TakeTurn(json_gamestate, _) => {
                    // TODO pass history after converting if we want to keep it
                    let move_ = self.client.get_move(&json_gamestate.to_common_game_state(), vec![])?;
                    self.send(ClientToServerMessage::Action(move_to_json_action(move_)))?;
                },
            }
        }
    }

    fn receive(&mut self) -> Option<ServerToClientMessage> {
        let mut de = Deserializer::from_reader(self.stream.try_clone().unwrap());
        util::try_with_timeout(self.timeout, || {
            ServerToClientMessage::deserialize(&mut de).ok()
        })
    }

    fn send(&mut self, message: ClientToServerMessage) -> Option<()> {
        self.stream.write(serde_json::to_string(&message).ok()?.as_bytes()).ok()?;
        Some(())
    }
}
