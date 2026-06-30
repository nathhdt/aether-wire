//! udp protocol module

use super::constants::UDP_HEADER_LENGTH_BYTES;

/// writes UDP header
#[allow(dead_code)]
pub fn write_udp_header(buf: &mut [u8], src_port: u16, dst_port: u16, payload_len: u16) {
    let length = UDP_HEADER_LENGTH_BYTES as u16 + payload_len;

    buf[0..2].copy_from_slice(&src_port.to_be_bytes());
    buf[2..4].copy_from_slice(&dst_port.to_be_bytes());
    buf[4..6].copy_from_slice(&length.to_be_bytes());
    buf[6..8].copy_from_slice(&0u16.to_be_bytes());
}
