use crate::common::action::{ Placement, Move, PlayerMove };
use crate::common::gamestate::GameState;
use crate::common::player::PlayerColor;
use crate::common::util;
use crate::server::client::Client;
use crate::server::message::*;

use std::net::TcpStream;
use std::time::Duration;
use std::io::{ Error, Write };

use serde::Deserialize;
use serde_json::Deserializer;

pub struct RemoteClient {
    stream: TcpStream,
    timeout: Duration,
}

impl RemoteClient {
    pub fn new(stream: TcpStream, timeout: Duration) -> RemoteClient {
        stream.set_read_timeout(Some(timeout)).unwrap();
        stream.set_write_timeout(Some(timeout)).unwrap();
        RemoteClient { stream, timeout }
    }

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
}

impl Client for RemoteClient {
    fn tournament_starting(&mut self) -> Option<()> {
        match self.call(start_message())? {
            ClientToServerMessage::Void(_) => Some(()),
            _ => None
        }
    }

    fn tournament_ending(&mut self, won: bool) -> Option<()> {
        match self.call(end_message(won))? {
            ClientToServerMessage::Void(_) => Some(()),
            _ => None
        }
    }

    fn initialize_game(&mut self, initial_gamestate: &GameState, player_color: PlayerColor) -> Option<()> {
        match self.call(playing_as_message(player_color))? {
            ClientToServerMessage::Void(_) => Some(()),
            _ => None
        }?;
        let other_colors = initial_gamestate.players.iter()
            .map(|player| player.1.color)
            .filter(|color| *color != player_color)
            .collect::<Vec<PlayerColor>>();
        match self.call(playing_with_message(&other_colors))? {
            ClientToServerMessage::Void(_) => Some(()),
            _ => None
        }
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
