use crate::client::player::PlayerInterface;
use crate::common::action::{Action, Placement, Move, PlayerMove};
use crate::common::util;
use crate::common::gamestate::GameState;
use crate::common::game_tree::GameTree;
use crate::common::player::PlayerColor;
use crate::server::message::{ ClientToServerMessage, ServerToClientMessage, serialize_gamestate, convert_to_json_actions };
use crate::client::strategy::{ Strategy, ZigZagMinMaxStrategy };

use std::net::TcpStream;
use std::time::Duration;
use std::io::{ Error, Write };
use std::net::ToSocketAddrs;
use std::thread;

use serde::Deserialize;
use serde_json::Deserializer;

pub struct ProxyClient {
    strategy: Box<dyn Strategy>,
    stream: TcpStream,
    timeout: Duration,
}

impl ProxyClient {
    pub fn new<A: ToSocketAddrs>(strategy: Box<dyn Strategy>, address: A, timeout: Duration) -> Option<ProxyClient> {
        let stream = TcpStream::connect(address).ok()?;
        stream.set_read_timeout(Some(timeout)).unwrap();
        stream.set_write_timeout(Some(timeout)).unwrap();
        Some(ProxyClient { strategy, stream, timeout })
    }

    pub fn tournament_loop(&mut self) -> Option<bool> {
        loop {
            match self.receive()? {
                ServerToClientMessage::Start(_) => {
                    println!("Starting tournament");
                    self.send(ClientToServerMessage::Void())?;
                },
                ServerToClientMessage::End((won,)) => {
                    println!("Ending tournament, won: {}", won);
                    self.send(ClientToServerMessage::Void())?;
                    return Some(won)
                },
                ServerToClientMessage::PlayingAs((color,)) => {
                    self.send(ClientToServerMessage::Void())?;
                },
                ServerToClientMessage::PlayingWith((other_colors,)) => {
                    self.send(ClientToServerMessage::Void())?;
                },
                ServerToClientMessage::Setup((JSONGameState,)) => {
                    // TODO
                    self.send(ClientToServerMessage::Void())?;
                },
                ServerToClientMessage::TakeTurn(JSONGameState, _) => {
                    // TODO
                    self.send(ClientToServerMessage::Void())?;
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
        // TODO
        Some(())
    } 
}