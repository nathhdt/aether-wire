//! IPv4 checksum protocol module

use super::utils::{fold, sum16};

/// computes IPv4 header checksum
pub fn header_checksum(header: &[u8]) -> u16 {
    fold(sum16(header, 0))
}
