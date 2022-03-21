use crate::network::network_client::NetworkClient;
use crate::network::proto::PacketReader;
use crate::{SResult, SimpleError};

/// Trying to read [handshake](https://wiki.vg/index.php?title=Protocol&oldid=14204#Handshake) packet
pub fn read_handshake_packet(client: &mut NetworkClient) -> SResult<(u32, String, u16, u32)> {
    // Reading bytes from client
    let (ok, p, err) = match client.read() {
        Ok((ok, p)) => (ok, Some(p), None),
        Err(err) => (false, None, Some(err)),
    };
    
    // Something went wrong
    if !ok || err.is_some() {
        return Err(SimpleError(
            String::from("Failed to read handshake packet"),
            if err.is_some() { err.unwrap().1 } else { None },
        ));
    }
    
    // Unwrapping Some(p) from line 9 back
    let mut p: Vec<u8> = p.unwrap();
    
    print!("real packet length: {} (", p.len());
    
    for v in p.iter() {print!("{},", v);}
    
    // Trying to read Length and PacketID from packet
    // (on handshaking stage PacketID must be equal to 0x00)
    let (length, packet_id) = p.read_base()?;
    
    println!(") stated packet length: {}, id = {}", length, packet_id);
    
    // Reading version, address and etc.
    let ver = p.get_varint()? as u32;
    let address = p.get_string()?;
    let port = p.get_u16();
    let next_state = p.get_varint()? as u32;
    
    // Protocol states can be only 1 - status, 2 - play
    if next_state >= 3 {
        return Err(SimpleError(String::from("Invalid client"), None));
    }
    
    // Returning unused bytes back into reader for next call
    if p.len() > 0 {
        client.return_unused(p);
    }
    
    Ok((ver, address, port, next_state))
}
