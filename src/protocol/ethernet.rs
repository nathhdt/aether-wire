//! ethernet protocol module

/// writes Ethernet header
#[allow(dead_code)]
pub fn write_ethernet_header(buf: &mut [u8], src_mac: [u8; 6], dst_mac: [u8; 6], ethertype: u16) {
    buf[0..6].copy_from_slice(&dst_mac);
    buf[6..12].copy_from_slice(&src_mac);
    buf[12..14].copy_from_slice(&ethertype.to_be_bytes());
}
