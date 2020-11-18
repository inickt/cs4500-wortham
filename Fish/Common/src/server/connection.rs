//! This module is a stub for future TcpStream related functionality
//! for connecting remote players. For such players it will define how
//! to read/write to communicate with a player which includes error
//! handling like a timeout of 30 seconds while waiting for each player message.
use std::net::TcpStream;
use std::time::Duration;

pub struct PlayerConnection {
    pub stream: TcpStream,
}

impl PlayerConnection {
    pub fn new(stream: TcpStream) -> PlayerConnection {
        let timeout = Duration::from_secs(30);
        stream.set_read_timeout(Some(timeout));
        stream.set_write_timeout(Some(timeout));
        PlayerConnection { stream }
    }
}