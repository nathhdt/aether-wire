//! network constants utilities module

/// interface link-layer types (linux/if_arp.h)
pub const ARPHRD_ETHER: u32 = 1;
pub const ARPHRD_PPP: u32 = 512;
pub const ARPHRD_IPGRE: u32 = 768;
pub const ARPHRD_IP6GRE: u32 = 769;
pub const ARPHRD_SIT: u32 = 776;
pub const ARPHRD_TUNNEL6: u32 = 778;
pub const ARPHRD_LOOPBACK: u32 = 772;
pub const ARPHRD_RAWIP: u32 = 65534;

/// interface operational state values (linux/if.h)
pub const IF_OPER_UP: u8 = 6;

/// address family values (linux/socket.h)
pub const AF_INET: u8 = 2;
pub const AF_INET6: u8 = 10;
