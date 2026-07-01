//! Netlink requests builder module

use super::{
    constants::{NLM_F_DUMP, NLM_F_REQUEST, RTM_GETADDR, RTM_GETLINK},
    types::{IfAddrMsg, IfInfoMsg, NlMsgHdr},
};

/// returns raw byte slice from POD structure
fn as_bytes<T: Copy>(value: &T) -> &[u8] {
    unsafe {
        core::slice::from_raw_parts(value as *const T as *const u8, core::mem::size_of::<T>())
    }
}

/// builds raw Netlink request from a header & body
fn build<T: Copy>(nlmsg_type: u16, nlmsg_flags: u16, body: T, seq: u32) -> Vec<u8> {
    let nlmsg_len = (NlMsgHdr::SIZE + core::mem::size_of::<T>()) as u32;

    let header = NlMsgHdr {
        nlmsg_len,
        nlmsg_type,
        nlmsg_flags,
        nlmsg_seq: seq,
        nlmsg_pid: 0,
    };

    let mut buf = Vec::with_capacity(nlmsg_len as usize);

    buf.extend_from_slice(as_bytes(&header));
    buf.extend_from_slice(as_bytes(&body));

    buf
}

/// builds RTM_GETLINK request for specific interface
pub fn build_getlink_request(ifindex: i32, seq: u32) -> Vec<u8> {
    build(
        RTM_GETLINK,
        NLM_F_REQUEST,
        IfInfoMsg {
            ifi_family: 0,
            ifi_pad: 0,
            ifi_type: 0,
            ifi_index: ifindex,
            ifi_flags: 0,
            ifi_change: 0,
        },
        seq,
    )
}

/// builds RTM_GETLINK dump request for all interfaces
pub fn build_getlink_dump_request(seq: u32) -> Vec<u8> {
    build(
        RTM_GETLINK,
        NLM_F_REQUEST | NLM_F_DUMP,
        IfInfoMsg {
            ifi_family: 0,
            ifi_pad: 0,
            ifi_type: 0,
            ifi_index: 0,
            ifi_flags: 0,
            ifi_change: 0,
        },
        seq,
    )
}

/// builds RTM_GETADDR dump request for all interface addresses
pub fn build_getaddr_dump_request(seq: u32) -> Vec<u8> {
    build(
        RTM_GETADDR,
        NLM_F_REQUEST | NLM_F_DUMP,
        IfAddrMsg {
            ifa_family: 0,
            ifa_prefixlen: 0,
            ifa_flags: 0,
            ifa_scope: 0,
            ifa_index: 0,
        },
        seq,
    )
}
