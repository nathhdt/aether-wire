//! Netlink constants module

/// generic Netlink message types
pub const NLMSG_ERROR: u16 = 2;
pub const NLMSG_DONE: u16 = 3;

/// Netlink message flags
pub const NLM_F_REQUEST: u16 = 0x0001;
pub const NLM_F_MULTI: u16 = 0x0002;

/// Netlink dump flags
pub const NLM_F_ROOT: u16 = 0x0100;
pub const NLM_F_MATCH: u16 = 0x0200;
pub const NLM_F_DUMP: u16 = NLM_F_ROOT | NLM_F_MATCH;

/// RTNetlink message types
pub const RTM_NEWLINK: u16 = 16;
pub const RTM_GETLINK: u16 = 18;

/// ifinfomsg attribute types
pub const IFLA_MTU: u16 = 4;
pub const IFLA_OPERSTATE: u16 = 16;
pub const IFLA_XDP: u16 = 43;
pub const IFLA_NUM_TX_QUEUES: u16 = 31;
pub const IFLA_NUM_RX_QUEUES: u16 = 32;

/// nested IFLA_XDP attribute types
pub const IFLA_XDP_ATTACHED: u16 = 2;
pub const IFLA_XDP_PROG_ID: u16 = 4;
pub const IFLA_XDP_FEATURES: u16 = 8;

/// XDP attachment state values
pub const XDP_ATTACHED_NONE: u8 = 0;
pub const XDP_ATTACHED_DRV: u8 = 1;
pub const XDP_ATTACHED_SKB: u8 = 2;
pub const XDP_ATTACHED_HW: u8 = 3;
pub const XDP_ATTACHED_MULTI: u8 = 4;

/// IFLA_XDP_FEATURES flag bits
pub const NETDEV_XDP_ACT_BASIC: u32 = 1 << 0;
pub const NETDEV_XDP_ACT_XSK_ZEROCOPY: u32 = 1 << 3;
pub const NETDEV_XDP_ACT_RX_SG: u32 = 1 << 5;

/// Netlink attribute type flags
pub const NLA_F_NESTED: u16 = 1 << 15;
pub const NLA_F_NET_BYTEORDER: u16 = 1 << 14;

/// base attribute type mask
pub const NLA_TYPE_MASK: u16 = !(NLA_F_NESTED | NLA_F_NET_BYTEORDER);
