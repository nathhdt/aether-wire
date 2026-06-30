//! ipv4 protocol module

use std::net::Ipv4Addr;

use super::checksum::ipv4;
use super::constants::{IP_DEFAULT_HOP_LIMIT, IPV4_HEADER_LENGTH_BYTES};

/// writes IPv4 header
#[allow(dead_code)]
pub fn write_ipv4_header(
    buf: &mut [u8],
    src: Ipv4Addr,
    dst: Ipv4Addr,
    payload_len: u16,
    protocol: u8,
) {
    let total_len = IPV4_HEADER_LENGTH_BYTES as u16 + payload_len;

    buf[0] = 0x45;
    buf[1] = 0x00;
    buf[2..4].copy_from_slice(&total_len.to_be_bytes());
    buf[4..6].copy_from_slice(&0u16.to_be_bytes());
    buf[6..8].copy_from_slice(&0x4000u16.to_be_bytes());
    buf[8] = IP_DEFAULT_HOP_LIMIT;
    buf[9] = protocol;
    buf[10..12].copy_from_slice(&0u16.to_be_bytes());
    buf[12..16].copy_from_slice(&src.octets());
    buf[16..20].copy_from_slice(&dst.octets());

    let csum = ipv4::header_checksum(&buf[0..IPV4_HEADER_LENGTH_BYTES]);
    buf[10..12].copy_from_slice(&csum.to_be_bytes());
}
