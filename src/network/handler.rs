use crate::network::network_client::ConnectionType::STATUS;
use crate::network::network_client::NetworkClient;
use crate::network::proto::packets::handshaking::read_handshake_packet;
use crate::network::proto::packets::status::read_status_packet;
use crate::SResult;
use mio::event::Event;

// Processing packet on handshaking stage
pub fn handshaking(conn: &mut NetworkClient, event: &Event) -> SResult<bool> {
    if !event.is_readable() { // there is not enough data
        return Ok(false);
    }
    
    let handshake = read_handshake_packet(conn);
    if handshake.is_err() { // dropping this connection
        return Ok(true);
    }
    
    let (_, _, _, next_state) = handshake.unwrap();
    
    // Changing protocol state, if needed
    conn.conn_type = match next_state {
        1 => STATUS,
        _ => STATUS,
    };
    Ok(false)
}

// Processing packet on status stage
pub fn status_handler(conn: &mut NetworkClient, event: &Event) -> SResult<bool> {
    // Checking if we can read and write
    if !event.is_readable() || !event.is_writable() {
        return Ok(false);
    }
    
    let status = read_status_packet(conn);
    
    Ok(status.is_err())
}
