use crate::server::client::Client;
use crate::server::message::*;
use crate::common::util;

use std::net::TcpStream;
use std::time::Duration;
use std::io::Write;

use serde::Deserialize;
use serde_json::Deserializer;

/// A remote interface from client -> server.
/// Communicates with the server via the internal TcpStream.
///
/// This proxy layer will not make any placement/move decisions
/// for itself, it simply delegates to the given Client for each
/// of these decisions. This way, you can have a remote ai or a
/// remote human player by passing in the appropriate client upon
/// construction. The server-side counterpart to this connection
/// would be the RemoteClient.
pub struct ClientToServerProxy {
    name: String,
    client: Box<dyn Client>,
    stream: TcpStream,
    timeout: Duration,
    player_count: usize,
}

impl ClientToServerProxy {
    pub fn new(name: String, client: Box<dyn Client>, address: &str, timeout: Duration) -> Option<ClientToServerProxy> {
        let stream = TcpStream::connect(address).ok()?;
        stream.set_read_timeout(Some(timeout)).unwrap();
        stream.set_write_timeout(Some(timeout)).unwrap();
        Some(ClientToServerProxy {
            name,
            client,
            stream,
            timeout,
            player_count: 0,
        })
    }

    /// Loops until the entire game is finished, forwarding each
    /// received message to the inner Client, returning early
    /// if any incoming message is malformed.
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
                ServerToClientMessage::PlayingWith((colors,)) => {
                    self.player_count = colors.len() + 1;
                    self.send(ClientToServerMessage::Void(JSONVoid::Void))?;
                },
                ServerToClientMessage::Setup((json_gamestate,)) => {
                    let gamestate = json_gamestate.to_common_game_state(self.player_count);
                    let placement = self.client.get_placement(&gamestate)?;
                    let json_position = placement_to_json_position(&gamestate.board, placement);
                    self.send(ClientToServerMessage::Position(json_position))?;
                },
                ServerToClientMessage::TakeTurn(json_gamestate, _) => {
                    let gamestate = json_gamestate.to_common_game_state(self.player_count);
                    let move_ = self.client.get_move(&gamestate, &[])?;
                    let json_move = move_to_json_action(&gamestate.board, move_);
                    self.send(ClientToServerMessage::Action(json_move))?;
                },
            }
        }
    }

    /// Send the client's name through the stream. The client's name is a bit special
    /// in that it is not a ClientToServerMessage since it could otherwise collide with
    /// the "void" message if the client names themselves "void".
    pub fn send_name(&mut self) -> Option<()> {
        let json_name = serde_json::to_string(&self.name).ok()?;
        self.stream.write(json_name.as_bytes()).ok()?;
        Some(())
    }

    /// Receive an arbitrary ServerToClientMessage from self.stream,
    /// waiting a maximum Duration of self.timeout
    fn receive(&mut self) -> Option<ServerToClientMessage> {
        let mut de = Deserializer::from_reader(self.stream.try_clone().unwrap());
        util::try_with_timeout(self.timeout, || {
            ServerToClientMessage::deserialize(&mut de).ok()
        })
    }

    /// Send an arbitrary ClientToServerMessage to self.stream
    fn send(&mut self, message: ClientToServerMessage) -> Option<()> {
        self.stream.write(serde_json::to_string(&message).ok()?.as_bytes()).ok()?;
        Some(())
    }
}
