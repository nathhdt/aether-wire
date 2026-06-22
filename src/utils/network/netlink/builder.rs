//! Netlink requests builder module

use crate::utils::network::netlink::{
    constants::{NLM_F_DUMP, NLM_F_REQUEST, RTM_GETADDR, RTM_GETLINK},
    types::{IfAddrMsg, IfInfoMsg, NlMsgHdr},
};

/// builds a raw Netlink request from a header and a ifinfomsg
fn build_request(nlmsg_type: u16, nlmsg_flags: u16, info: IfInfoMsg, seq: u32) -> Vec<u8> {
    let nlmsg_len = (NlMsgHdr::SIZE + IfInfoMsg::SIZE) as u32;

    let header = NlMsgHdr {
        nlmsg_len,
        nlmsg_type,
        nlmsg_flags,
        nlmsg_seq: seq,
        nlmsg_pid: 0,
    };

    let mut buf = Vec::with_capacity(nlmsg_len as usize);

    unsafe {
        let header_bytes =
            core::slice::from_raw_parts(&header as *const NlMsgHdr as *const u8, NlMsgHdr::SIZE);
        let info_bytes =
            core::slice::from_raw_parts(&info as *const IfInfoMsg as *const u8, IfInfoMsg::SIZE);

        buf.extend_from_slice(header_bytes);
        buf.extend_from_slice(info_bytes);
    }

    buf
}

/// builds a raw Netlink request from a header and a ifaddrmsg
fn build_addr_request(nlmsg_type: u16, nlmsg_flags: u16, addr: IfAddrMsg, seq: u32) -> Vec<u8> {
    let nlmsg_len = (NlMsgHdr::SIZE + IfAddrMsg::SIZE) as u32;

    let header = NlMsgHdr {
        nlmsg_len,
        nlmsg_type,
        nlmsg_flags,
        nlmsg_seq: seq,
        nlmsg_pid: 0,
    };

    let mut buf = Vec::with_capacity(nlmsg_len as usize);

    unsafe {
        let header_bytes =
            core::slice::from_raw_parts(&header as *const NlMsgHdr as *const u8, NlMsgHdr::SIZE);
        let addr_bytes =
            core::slice::from_raw_parts(&addr as *const IfAddrMsg as *const u8, IfAddrMsg::SIZE);

        buf.extend_from_slice(header_bytes);
        buf.extend_from_slice(addr_bytes);
    }

    buf
}

/// builds a RTM_GETLINK request for a specific interface by index
pub fn build_getlink_request(ifindex: i32, seq: u32) -> Vec<u8> {
    build_request(
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

/// builds a RTM_GETLINK dump request for all interfaces
pub fn build_getlink_dump_request(seq: u32) -> Vec<u8> {
    build_request(
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

/// builds a RTM_GETADDR dump request for all interface addresses
pub fn build_getaddr_dump_request(seq: u32) -> Vec<u8> {
    build_addr_request(
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
