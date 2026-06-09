//! Netlink types and structures

/// Netlink message header
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct NlMsgHdr {
    pub nlmsg_len: u32,
    pub nlmsg_type: u16,
    pub nlmsg_flags: u16,
    pub nlmsg_seq: u32,
    pub nlmsg_pid: u32,
}

impl NlMsgHdr {
    pub const SIZE: usize = core::mem::size_of::<Self>();
}

/// interface information message
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct IfInfoMsg {
    pub ifi_family: u8,
    pub ifi_pad: u8,
    pub ifi_type: u16,
    pub ifi_index: i32,
    pub ifi_flags: u32,
    pub ifi_change: u32,
}

impl IfInfoMsg {
    pub const SIZE: usize = core::mem::size_of::<Self>();
}

/// routing attribute
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct RtAttr {
    pub rta_len: u16,
    pub rta_type: u16,
}

impl RtAttr {
    pub const SIZE: usize = core::mem::size_of::<Self>();
}

/// Netlink message error
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct NlMsgErr {
    pub error: i32,
}

impl NlMsgErr {
    pub const SIZE: usize = core::mem::size_of::<Self>();
}
