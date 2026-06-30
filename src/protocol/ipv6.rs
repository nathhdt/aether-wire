//! ipv6 protocol module

use std::net::Ipv6Addr;

use super::constants::IP_DEFAULT_HOP_LIMIT;

/// writes IPv6 header
#[allow(dead_code)]
pub fn write_ipv6_header(
    buf: &mut [u8],
    src: Ipv6Addr,
    dst: Ipv6Addr,
    payload_len: u16,
    next_header: u8,
) {
    buf[0..4].copy_from_slice(&0x6000_0000u32.to_be_bytes());
    buf[4..6].copy_from_slice(&payload_len.to_be_bytes());
    buf[6] = next_header;
    buf[7] = IP_DEFAULT_HOP_LIMIT;
    buf[8..24].copy_from_slice(&src.octets());
    buf[24..40].copy_from_slice(&dst.octets());
}
