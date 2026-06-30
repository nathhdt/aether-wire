//! constants protocol module

// aether-wire
pub const AW_HEADER_LENGTH_BYTES: u16 = 26;

// UDP
pub const UDP_HEADER_LENGTH_BYTES: usize = 8;
pub const IPV4_UDP_MAX_PAYLOAD_LENGTH_BYTES: u16 = 65_507;
pub const IPV6_UDP_MAX_PAYLOAD_LENGTH_BYTES: u16 = 65_487;

// IP
pub const IPV4_HEADER_LENGTH_BYTES: usize = 20;
pub const IP_PROTO_UDP: u8 = 17;
pub const IP_DEFAULT_HOP_LIMIT: u8 = 64;

// Ethernet
pub const ETHERNET_IPV4_UDP_OVERHEAD_BYTES: u32 = 42;
pub const ETHERNET_IPV6_UDP_OVERHEAD_BYTES: u32 = 62;
