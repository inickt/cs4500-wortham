//! This module is a stub for future TcpStream related functionality
//! for connecting remote players. For such players it will define how
//! to read/write to communicate with a player which includes error
//! handling like a timeout of 1 minute while waiting for each player message.
use serde_json::{ Deserializer, de::IoRead };

use std::io::{ Read, Write };

/// TODO: Cases to implement: 
/// - timeout
/// - tcp streams
pub struct PlayerConnection {
    pub input_deserializer: Deserializer<IoRead<Box<dyn Read>>>,
    pub output_stream: Box<dyn Write>,
}

impl PlayerConnection {
    pub fn new(input_stream: Box<dyn Read>, output_stream: Box<dyn Write>) -> PlayerConnection {
        let input_deserializer = Deserializer::from_reader(input_stream);
        PlayerConnection {
            input_deserializer,
            output_stream
        }
    }
}
