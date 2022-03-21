use crate::{SResult, SimpleError};
use mio::net::TcpStream;
use std::io::{ErrorKind, Read};

// Minecraft protocol states
pub enum ConnectionType {
    HANDSHAKING,
    STATUS,
}

pub struct NetworkClient {
    pub stream: TcpStream,
    pub conn_type: ConnectionType,
    pub unused_buffer: Option<Vec<u8>>
}

impl NetworkClient {
    // Function for reading input bytes
    pub fn read(&mut self) -> SResult<(bool, Vec<u8>)> {
        // if there was some data unused in previous read, returning them again
        if let Some(buffer) = self.unused_buffer.take() {
            println!("Using buffer rather than reading data from network");
            return Ok((true, buffer));
        }
        
        let mut bytes = vec![0; 4096];
        let (ok, err) = match self.stream.read(&mut bytes) {
            // Connection was closed, returning flag that we shall drop this socket
            Ok(0) => (false, None),
            // There was some data
            Ok(n) => {
                bytes.resize(n, 0); // Shrinking buffer
                (true, None)
            }
            // There is no data in socket
            Err(ref err) if err.kind() == ErrorKind::WouldBlock => (false, None),
            // Failed to read
            Err(err) => (false, Some(err)),
        };
        
        // Converting network error to SimpleError
        if err.is_some() {
            return Err(SimpleError(String::from("Failed to read packet"), err));
        }
        
        Ok((ok, bytes))
    }
    
    pub fn return_unused(&mut self, buffer: Vec<u8>) {
        // ownership on buffer is transferred here
        assert!(self.unused_buffer.is_none());
        info!("Buffer of size {} was returned to NetworkClient", buffer.len());
        self.unused_buffer.insert(buffer);
    }
}
