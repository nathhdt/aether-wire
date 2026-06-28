//! network interfaces Netlink parsing utilities module

use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use crate::utils::network::interfaces::constants::{AF_INET, AF_INET6};
use crate::utils::network::interfaces::types::{Interface, InterfaceAddress};
use crate::utils::network::netlink::{
    constants::{
        IFA_ADDRESS, IFA_LOCAL, IFLA_MTU, IFLA_NUM_RX_QUEUES, IFLA_NUM_TX_QUEUES, IFLA_OPERSTATE,
        IFLA_XDP, IFLA_XDP_ATTACHED, IFLA_XDP_FEATURES, IFLA_XDP_PROG_ID,
    },
    parser::{RtAttrIter, parse_ifinfomsg},
    types::IfAddrMsg,
};

/// extracts the ifindex from a RTM_NEWLINK payload
pub fn extract_netlink_ifindex(payload: &[u8]) -> Option<i32> {
    let (ifinfo, _) = parse_ifinfomsg(payload)?;
    Some(ifinfo.ifi_index)
}

/// extracts the ifindex from a RTM_NEWADDR payload
pub fn extract_netlink_addr_ifindex(payload: &[u8]) -> Option<i32> {
    if payload.len() < IfAddrMsg::SIZE {
        return None;
    }

    let msg: IfAddrMsg = unsafe { core::ptr::read_unaligned(payload.as_ptr() as *const IfAddrMsg) };
    Some(msg.ifa_index as i32)
}

/// parses a RTM_NEWADDR payload into an Interface
pub fn parse_netlink_interface_address(payload: &[u8], interface: &mut Interface) {
    if payload.len() < IfAddrMsg::SIZE {
        return;
    }

    let msg: IfAddrMsg = unsafe { core::ptr::read_unaligned(payload.as_ptr() as *const IfAddrMsg) };

    if msg.ifa_index as i32 != interface.index {
        return;
    }

    let attrs = &payload[IfAddrMsg::SIZE..];

    // IFA_LOCAL for IPv4, IFA_ADDRESS for IPv6
    let target_attr = if msg.ifa_family == AF_INET6 {
        IFA_ADDRESS
    } else {
        IFA_LOCAL
    };

    for (attr_type, attr_data) in RtAttrIter::new(attrs) {
        if attr_type != target_attr {
            continue;
        }

        let addr = if msg.ifa_family == AF_INET && attr_data.len() >= 4 {
            let Ok(bytes) = <[u8; 4]>::try_from(&attr_data[..4]) else {
                continue;
            };
            IpAddr::V4(Ipv4Addr::from(bytes))
        } else if msg.ifa_family == AF_INET6 && attr_data.len() >= 16 {
            let Ok(bytes) = <[u8; 16]>::try_from(&attr_data[..16]) else {
                continue;
            };
            IpAddr::V6(Ipv6Addr::from(bytes))
        } else {
            continue;
        };

        interface.addresses.push(InterfaceAddress {
            addr,
            prefix_len: msg.ifa_prefixlen,
        });

        return;
    }
}

/// parses a RTM_NEWLINK payload into an Interface
pub fn parse_netlink_interface_details(payload: &[u8], interface: &mut Interface) {
    let Some((_, attrs)) = parse_ifinfomsg(payload) else {
        return;
    };

    for (attr_type, attr_data) in RtAttrIter::new(attrs) {
        match attr_type {
            t if t == IFLA_OPERSTATE && !attr_data.is_empty() => {
                interface.operstate = Some(attr_data[0]);
            }
            t if t == IFLA_XDP => {
                for (xdp_type, xdp_data) in RtAttrIter::new(attr_data) {
                    if xdp_type == IFLA_XDP_FEATURES
                        && xdp_data.len() >= 8
                        && let Ok(bytes) = xdp_data[..8].try_into()
                    {
                        interface.xdp_features = Some(u64::from_ne_bytes(bytes));
                    }

                    if xdp_type == IFLA_XDP_ATTACHED && !xdp_data.is_empty() {
                        interface.xdp_attached = Some(xdp_data[0]);
                    }

                    if xdp_type == IFLA_XDP_PROG_ID
                        && xdp_data.len() >= 4
                        && let Ok(bytes) = xdp_data[..4].try_into()
                    {
                        interface.xdp_prog_id = Some(u32::from_ne_bytes(bytes));
                    }
                }
            }
            t if t == IFLA_MTU && attr_data.len() >= 4 => {
                if let Ok(bytes) = attr_data[..4].try_into() {
                    interface.mtu = Some(u32::from_ne_bytes(bytes));
                }
            }
            t if t == IFLA_NUM_RX_QUEUES && attr_data.len() >= 4 => {
                if let Ok(bytes) = attr_data[..4].try_into() {
                    interface.rx_queues = Some(u32::from_ne_bytes(bytes));
                }
            }
            t if t == IFLA_NUM_TX_QUEUES && attr_data.len() >= 4 => {
                if let Ok(bytes) = attr_data[..4].try_into() {
                    interface.tx_queues = Some(u32::from_ne_bytes(bytes));
                }
            }
            _ => {}
        }
    }
}
