//! sends messages to the control channel session

use anyhow::Result;
use std::io::{Read, Write};

use crate::bail_error;
use crate::protocol::messages::Message;

// maximum message size
const MAX_MSG_SIZE: u32 = 1024 * 1024;

// serializes a message and sends it to the control channel session
pub fn send_message<W: Write>(stream: &mut W, msg: &Message) -> Result<()> {
    // payload serialization
    let payload = bincode::serialize(msg)?;

    if payload.len() as u32 > MAX_MSG_SIZE {
        bail_error!("ctrl", "message too large : {} bytes", payload.len());
    }

    // encoded payload length (RFC 1700)
    let len_bytes = (payload.len() as u32).to_be_bytes();

    // sends the payload size then the payload itself
    stream.write_all(&len_bytes)?;
    stream.write_all(&payload)?;

    Ok(())
}

// reads a received payload from the control channel session
pub fn read_message<R: Read>(stream: &mut R) -> Result<Message> {
    // reads the received payload size
    let mut len_bytes = [0u8; 4];
    stream.read_exact(&mut len_bytes)?;

    let len = u32::from_be_bytes(len_bytes); // big endian

    if len > MAX_MSG_SIZE {
        bail_error!("ctrl", "announced message too large : {} bytes", len);
    }

    // payload buffer
    let mut payload = vec![0u8; len as usize];
    stream.read_exact(&mut payload)?;

    // message deserialization
    let msg = bincode::deserialize(&payload)?;

    Ok(msg)
}
