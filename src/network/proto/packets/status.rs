use crate::config::PROTOCOL_VERSION;
use crate::network::proto::packet_read::PacketReader;
use crate::network::proto::packet_write::PacketWriter;
use crate::network::network_client::NetworkClient;
use crate::utils::chat::ChatMessage;
use crate::{SResult, SimpleError};
use std::io::Write;

// Structs for status MOTD response
#[derive(Debug, Serialize)]
pub struct ListPingResponse {
    pub version: ListPingResponseVersion,
    pub players: ListPingResponsePlayers,
    pub description: ChatMessage,
}

#[derive(Debug, Serialize)]
pub struct ListPingResponseVersion {
    pub name: String,
    pub protocol: u32,
}

#[derive(Debug, Serialize)]
pub struct ListPingResponsePlayers {
    pub max: u32,
    pub online: u32,
    pub sample: Vec<ListPingResponsePlayerSample>,
}

#[derive(Debug, Serialize)]
pub struct ListPingResponsePlayerSample {
    pub name: String,
    pub id: String,
}
/// Build packet's bytes as result
pub fn create_server_list_ping_response() -> Vec<u8> {
    // Initialize empty byte's vector
    let mut bytes = Vec::new();
    // Generating String and convert to bytes.
    // String generated as JSON by serde and serde_json libraries
    bytes.write_string(
        serde_json::to_string(&ListPingResponse {
            version: ListPingResponseVersion {
                name: String::from("ULE"),
                protocol: PROTOCOL_VERSION,
            },
            players: ListPingResponsePlayers {
                max: 10,
                online: 0,
                sample: vec![],
            },
            // Some clients can read colors and so on without convert into JSON
            description: ChatMessage::str("&a&lHello!"),
        })
        .unwrap(),
    );
    // Build completed packet. Server List Ping - PacketID is 0x00
    bytes.create_packet(0x00)
}

/// Trying to read status packet
pub fn read_status_packet(client: &mut NetworkClient) -> SResult<()> {
    // Reading bytes from client
    let (ok, p, err) = match client.read() {
        Ok((ok, p)) => (ok, Some(p), None),
        Err(err) => (false, None, Some(err)),
    };
    
    // Something went wrong
    if !ok {
        return Err(SimpleError(
            String::from("Failed to read status packet"),
            if err.is_some() { err.unwrap().1 } else { None },
        ));
    }
    
    // Unwrapping read data (Some(p)) from line 61 back
    let mut p: Vec<u8> = p.unwrap();
    
    print!("real packet length: {} (", p.len());
    
    for v in p.iter() {print!("{},", v);}
    
    // Trying to read Length and PacketID from packet
    // (on status stage PacketID must be equal to 0x00 or 0x01)
    let (_, packet_id) = p.read_base()?;
    
    match packet_id {
        0x00 => { // Ping List
            // drop(bytes); - will be executed automatically
            client.stream.write_all(&*create_server_list_ping_response());
        }
        0x01 => { // Ping-Pong
            let ping_data = p.get_i64();
            
            client.stream.write_all(b"\x01");
            client.stream.write_all(&i64::to_be_bytes(ping_data));
            
            match client.stream.peer_addr() {
                Ok(v) => {
                    info!("Server was pinged from {}", v)
                },
                Err(_) => {
                    info!("Server was pinged from somewhere.")
                }
            }
        }
        _ => {}
    };
    
    if p.len() > 0 {
        client.return_unused(p);
    }
    
    Ok(())
}
