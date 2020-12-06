use crate::server::client::Client;
use crate::server::message::*;
use crate::common::util;
use crate::common::gamestate::GameState;

use std::net::TcpStream;
use std::time::Duration;
use std::io::Write;

use serde::Deserialize;
use serde_json::Deserializer;

pub struct ClientToServerProxy {
    name: String,
    client: Box<dyn Client>,
    stream: TcpStream,
    timeout: Duration,
    state: Option<GameState>,
}

impl ClientToServerProxy {
    pub fn new(name: String, client: Box<dyn Client>, address: &str, timeout: Duration) -> Option<ClientToServerProxy> {
        let stream = TcpStream::connect(address).ok()?;
        stream.set_read_timeout(Some(timeout)).unwrap();
        stream.set_write_timeout(Some(timeout)).unwrap();
        Some(ClientToServerProxy { name, client, stream, timeout, state: None })
    }

    // TODO: Add tests
    pub fn tournament_loop(&mut self) -> Option<bool> {
        self.send_name()?;
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
                ServerToClientMessage::PlayingAs(_) => {
                    self.send(ClientToServerMessage::Void(JSONVoid::Void))?;
                },
                ServerToClientMessage::PlayingWith(_) => {
                    self.send(ClientToServerMessage::Void(JSONVoid::Void))?;
                },
                ServerToClientMessage::Setup((json_gamestate,)) => {
                    let gamestate = json_gamestate.to_common_game_state(self.state.as_ref());
                    let placement = self.client.get_placement(&gamestate)?;
                    let json_position = placement_to_json_position(&gamestate.board, placement);
                    self.state = Some(gamestate);
                    self.send(ClientToServerMessage::Position(json_position))?;
                },
                ServerToClientMessage::TakeTurn(json_gamestate, _) => {
                    // TODO pass history after converting if we want to keep it
                    let gamestate = json_gamestate.to_common_game_state(self.state.as_ref());
                    let move_ = self.client.get_move(&gamestate, &[])?;
                    let json_move = move_to_json_action(&gamestate.board, move_);
                    self.state = Some(gamestate);
                    self.send(ClientToServerMessage::Action(json_move))?;
                },
            }
        }
    }

    fn send_name(&mut self) -> Option<()> {
        let json_name = serde_json::to_string(&self.name).ok()?;
        self.stream.write(json_name.as_bytes()).ok()?;
        Some(())
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
