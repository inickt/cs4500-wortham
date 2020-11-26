//! This module is a stub for future TcpStream related functionality
//! for connecting remote players. For such players it will define how
//! to read/write to communicate with a player which includes error
//! handling like a timeout of 30 seconds while waiting for each player message.
use crate::common::action::Action;

use std::net::{ TcpListener, TcpStream };
use std::time::{ Duration, Instant };
use std::io::{ Write, Error, ErrorKind };

use serde::Deserialize;
use serde_json::Deserializer;

const TIMEOUT: Duration = Duration::from_secs(30);

pub struct PlayerConnection {
    pub listener: TcpListener,
    stream: Option<TcpStream>,
}

impl PlayerConnection {
    pub fn new(listener: TcpListener) -> PlayerConnection {
        listener.set_nonblocking(true).unwrap();
        PlayerConnection { listener, stream: None }
    }

    pub fn receive_action(&mut self) -> Option<Action> {
        let mut de = Deserializer::from_reader(self.connect_stream()?.try_clone().unwrap());
        PlayerConnection::try_with_timeout(|| {
            Action::deserialize(&mut de).ok()
        })
    }

    pub fn write(&mut self, bytes: &[u8]) -> Result<usize, Error> {
        match self.connect_stream() {
            Some(stream) => stream.write(bytes),
            None => Err(Error::new(ErrorKind::Other, "Unable to connect to stream")),
        }
    }

    fn connect_stream(&mut self) -> Option<&mut TcpStream> {
        if self.stream.is_none() {
            let stream = self.wait_for_connection()?;
            stream.set_read_timeout(Some(TIMEOUT)).unwrap();
            stream.set_write_timeout(Some(TIMEOUT)).unwrap();
            self.stream = Some(stream);
        }
        self.stream.as_mut()
    }
    
    fn wait_for_connection(&self) -> Option<TcpStream> {
        PlayerConnection::try_with_timeout(|| {
            self.listener.accept().ok().map(|pair| pair.0)
        })
    }

    fn try_with_timeout<F, U>(mut f: F) -> Option<U>
        where F: FnMut() -> Option<U>
    {
        let start_time = Instant::now();
        loop {
            match f() {
                Some(value) => return Some(value),
                None if start_time.elapsed() < TIMEOUT => continue,
                _ => return None,
            }
        }
    }
}