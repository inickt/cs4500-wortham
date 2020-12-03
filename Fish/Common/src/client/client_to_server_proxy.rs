use crate::server::client::Client;
use crate::common::action::{ Placement, Move, PlayerMove };
use crate::common::util;
use crate::common::gamestate::GameState;
use crate::common::game_tree::GameTree;
use crate::common::player::PlayerColor;
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
        loop {
            match self.receive()? {
                // TODO use client
                // TODO add name
                ServerToClientMessage::Start(_) => {
                    // println!("Starting tournament");
                    self.send(ClientToServerMessage::Void(JSONVoid::Void))?;
                },
                ServerToClientMessage::End((won,)) => {
                    println!("Ending tournament, won: {}", won);
                    self.send(ClientToServerMessage::Void(JSONVoid::Void))?;
                    return Some(won)
                },
                ServerToClientMessage::PlayingAs((color,)) => {
                    self.send(ClientToServerMessage::Void(JSONVoid::Void))?;
                },
                ServerToClientMessage::PlayingWith((other_colors,)) => {
                    self.send(ClientToServerMessage::Void(JSONVoid::Void))?;
                },
                ServerToClientMessage::Setup((JSONGameState,)) => {
                    // TODO
                    self.send(ClientToServerMessage::Void(JSONVoid::Void))?;
                },
                ServerToClientMessage::TakeTurn(JSONGameState, _) => {
                    // TODO
                    self.send(ClientToServerMessage::Void(JSONVoid::Void))?;
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