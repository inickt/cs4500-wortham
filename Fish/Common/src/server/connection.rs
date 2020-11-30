//! This module is a stub for future TcpStream related functionality
//! for connecting remote players. For such players it will define how
//! to read/write to communicate with a player which includes error
//! handling like a timeout of 30 seconds while waiting for each player message.
use crate::common::action::Action;
use crate::common::util;

use std::net::TcpStream;
use std::time::Duration;
use std::io::{ Error, Write };

use serde::Deserialize;
use serde_json::Deserializer;

const TIMEOUT: Duration = Duration::from_secs(30);

pub struct PlayerConnection {
    pub stream: TcpStream,
}

impl PlayerConnection {
    pub fn new(stream: TcpStream) -> PlayerConnection {
        PlayerConnection::new_with_timeout(stream, TIMEOUT)
    }

    pub fn new_with_timeout(stream: TcpStream, timeout: Duration) -> PlayerConnection {
        stream.set_read_timeout(Some(timeout)).unwrap();
        stream.set_write_timeout(Some(timeout)).unwrap();
        PlayerConnection { stream }
    }

    pub fn receive<'a, T: Deserialize<'a>>(&mut self) -> Option<T> {
        let mut de = Deserializer::from_reader(self.stream.try_clone().unwrap());
        util::try_with_timeout(TIMEOUT, || {
            T::deserialize(&mut de).ok()
        })
    }

    pub fn receive_action(&mut self) -> Option<Action> {
        self.receive()
    }

    /// Receive and verify a player name. A name must:
    /// - consist of only ascii alphabetic characters
    /// - be at least 1 and at most 12 characters in length
    pub fn receive_name(&mut self) -> Option<String> {
        let name: String = self.receive()?;
        if !name.is_empty() && name.len() <= 12 && name.chars().all(|c| c.is_ascii_alphabetic()) {
            Some(name)
        } else {
            None
        }
    }

    pub fn write(&mut self, bytes: &[u8]) -> Result<usize, Error> {
        self.stream.write(bytes)
    }
}
