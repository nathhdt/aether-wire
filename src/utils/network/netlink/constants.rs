//! Netlink constants

/// generic Netlink message types
pub const NLMSG_NOOP: u16 = 1;
pub const NLMSG_ERROR: u16 = 2;
pub const NLMSG_DONE: u16 = 3;
pub const NLMSG_OVERRUN: u16 = 4;

/// Netlink message flags
pub const NLM_F_REQUEST: u16 = 0x0001;
pub const NLM_F_MULTI: u16 = 0x0002;
pub const NLM_F_ACK: u16 = 0x0004;

pub const NLM_F_ROOT: u16 = 0x0100;
pub const NLM_F_MATCH: u16 = 0x0200;
pub const NLM_F_DUMP: u16 = NLM_F_ROOT | NLM_F_MATCH;

/// RTNetlink message types
pub const RTM_NEWLINK: u16 = 16;
pub const RTM_DELLINK: u16 = 17;
pub const RTM_GETLINK: u16 = 18;

/// ifinfomsg attributes
pub const IFLA_UNSPEC: u16 = 0;
pub const IFLA_ADDRESS: u16 = 1;
pub const IFLA_BROADCAST: u16 = 2;
pub const IFLA_IFNAME: u16 = 3;
pub const IFLA_MTU: u16 = 4;
pub const IFLA_LINK: u16 = 5;
pub const IFLA_QDISC: u16 = 6;
pub const IFLA_STATS: u16 = 7;
pub const IFLA_OPERSTATE: u16 = 16;
pub const IFLA_LINKMODE: u16 = 17;
pub const IFLA_XDP: u16 = 43;

/// IFLA_XDP nested attributes
pub const IFLA_XDP_UNSPEC: u16 = 0;
pub const IFLA_XDP_FD: u16 = 1;
pub const IFLA_XDP_ATTACHED: u16 = 2;
pub const IFLA_XDP_PROG_ID: u16 = 3;
pub const IFLA_XDP_FLAGS: u16 = 4;
pub const IFLA_XDP_DRV_PROG_ID: u16 = 5;
pub const IFLA_XDP_SKB_PROG_ID: u16 = 6;
pub const IFLA_XDP_HW_PROG_ID: u16 = 7;
pub const IFLA_XDP_FEATURES: u16 = 8;

/// XDP attachment modes
pub const XDP_ATTACHED_NONE: u8 = 0;
pub const XDP_ATTACHED_DRV: u8 = 1;
pub const XDP_ATTACHED_SKB: u8 = 2;
pub const XDP_ATTACHED_HW: u8 = 3;
pub const XDP_ATTACHED_MULTI: u8 = 4;

/// IFLA_XDP_FEATURES bitmask
pub const NETDEV_XDP_ACT_BASIC: u32 = 1 << 0;
pub const NETDEV_XDP_ACT_REDIRECT: u32 = 1 << 1;
pub const NETDEV_XDP_ACT_NDO_XMIT: u32 = 1 << 2;
pub const NETDEV_XDP_ACT_XSK_ZEROCOPY: u32 = 1 << 3;
pub const NETDEV_XDP_ACT_HW_OFFLOAD: u32 = 1 << 4;
pub const NETDEV_XDP_ACT_RX_SG: u32 = 1 << 5;
