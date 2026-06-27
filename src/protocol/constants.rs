//! constants protocol module

/// aether-wire UDP payload minimum length
pub const AW_UDP_PAYLOAD_MIN_LENGTH_BYTES: u16 = 18;

/// maximum UDP payload length over IPv4
pub const IPV4_UDP_MAX_PAYLOAD_LENGTH_BYTES: u16 = 65_507;

/// maximum UDP payload length over IPv6
pub const IPV6_UDP_MAX_PAYLOAD_LENGTH_BYTES: u16 = 65_487;

/// Ethernet IPv4 UDP header overhead
pub const ETHERNET_IPV4_UDP_OVERHEAD_BYTES: u32 = 42;

/// Ethernet IPv6 UDP header overhead
pub const ETHERNET_IPV6_UDP_OVERHEAD_BYTES: u32 = 62;
