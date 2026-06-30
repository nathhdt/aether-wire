//! aether-wire protocol module

use super::constants::AW_HEADER_LENGTH_BYTES;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AetherWireHeader {
    pub stream_id: u16,
    pub seq: u64,
    pub ref_seq: u64,
    pub ref_tx_timestamp_ns: u64,
}

impl AetherWireHeader {
    #[allow(dead_code)]
    pub const SIZE: usize = AW_HEADER_LENGTH_BYTES as usize;

    #[allow(dead_code)]
    pub fn write(&self, buf: &mut [u8]) {
        buf[0..2].copy_from_slice(&self.stream_id.to_be_bytes());
        buf[2..10].copy_from_slice(&self.seq.to_be_bytes());
        buf[10..18].copy_from_slice(&self.ref_seq.to_be_bytes());
        buf[18..26].copy_from_slice(&self.ref_tx_timestamp_ns.to_be_bytes());
    }

    #[allow(dead_code)]
    pub fn read(buf: &[u8]) -> Option<Self> {
        Some(Self {
            stream_id: u16::from_be_bytes(buf.get(0..2)?.try_into().ok()?),
            seq: u64::from_be_bytes(buf.get(2..10)?.try_into().ok()?),
            ref_seq: u64::from_be_bytes(buf.get(10..18)?.try_into().ok()?),
            ref_tx_timestamp_ns: u64::from_be_bytes(buf.get(18..26)?.try_into().ok()?),
        })
    }
}
