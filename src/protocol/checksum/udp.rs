//! UDP checksum protocol module

use std::net::{Ipv4Addr, Ipv6Addr};

use crate::protocol::constants::IP_PROTO_UDP;

use super::utils::{finalize_udp, sum16};

/// computes UDP checksum over IPv4 pseudo-header
#[allow(dead_code)]
pub fn ipv4_checksum(src: Ipv4Addr, dst: Ipv4Addr, segment: &[u8]) -> u16 {
    let mut acc = sum16(&src.octets(), 0);
    acc = sum16(&dst.octets(), acc);
    acc += IP_PROTO_UDP as u32;
    acc += segment.len() as u32;

    finalize_udp(sum16(segment, acc))
}

/// computes UDP checksum over IPv6 pseudo-header
#[allow(dead_code)]
pub fn ipv6_checksum(src: Ipv6Addr, dst: Ipv6Addr, segment: &[u8]) -> u16 {
    let mut acc = sum16(&src.octets(), 0);
    acc = sum16(&dst.octets(), acc);
    acc += segment.len() as u32;
    acc += IP_PROTO_UDP as u32;

    finalize_udp(sum16(segment, acc))
}
