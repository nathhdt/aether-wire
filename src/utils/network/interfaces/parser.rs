//! network interfaces parsing utilities module

use std::{
    fs,
    net::{IpAddr, Ipv4Addr, Ipv6Addr},
    path::Path,
};

use crate::utils::network::netlink::{
    constants::{
        IFA_ADDRESS, IFA_LOCAL, IFLA_MTU, IFLA_NUM_RX_QUEUES, IFLA_NUM_TX_QUEUES, IFLA_OPERSTATE,
        IFLA_XDP, IFLA_XDP_ATTACHED, IFLA_XDP_FEATURES, IFLA_XDP_PROG_ID,
    },
    parser::{RtAttrIter, parse_ifinfomsg},
    types::IfAddrMsg,
};

use super::constants::{
    AF_INET, AF_INET6, ARPHRD_ETHER, ARPHRD_IP6GRE, ARPHRD_IPGRE, ARPHRD_LOOPBACK, ARPHRD_PPP,
    ARPHRD_RAWIP, ARPHRD_SIT, ARPHRD_TUNNEL6,
};
use super::types::{Interface, InterfaceAddress, InterfaceClass, InterfaceKind};

/// parses a RTM_NEWADDR payload, returns ifindex if an address was pushed
pub fn parse_netlink_interface_address(payload: &[u8], interface: &mut Interface) -> Option<i32> {
    if payload.len() < IfAddrMsg::SIZE {
        return None;
    }

    let msg: IfAddrMsg = unsafe { core::ptr::read_unaligned(payload.as_ptr() as *const IfAddrMsg) };

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
            let bytes: [u8; 4] = attr_data[..4].try_into().ok()?;
            IpAddr::V4(Ipv4Addr::from(bytes))
        } else if msg.ifa_family == AF_INET6 && attr_data.len() >= 16 {
            let bytes: [u8; 16] = attr_data[..16].try_into().ok()?;
            IpAddr::V6(Ipv6Addr::from(bytes))
        } else {
            continue;
        };

        interface.addresses.push(InterfaceAddress {
            addr,
            prefix_len: msg.ifa_prefixlen,
        });

        return Some(msg.ifa_index as i32);
    }

    None
}

/// parses interface class
pub fn parse_interface_class(path: &Path) -> InterfaceClass {
    if path.join("device").exists() {
        InterfaceClass::Device
    } else {
        InterfaceClass::Virtual
    }
}

/// extracts the ifindex from a RTM_NEWLINK payload
pub fn parse_netlink_ifindex(payload: &[u8]) -> Option<i32> {
    let (ifinfo, _) = parse_ifinfomsg(payload)?;
    Some(ifinfo.ifi_index)
}

/// parses a RTM_NEWLINK payload into an interface
pub fn parse_netlink_interface_details(payload: &[u8], interface: &mut Interface) -> Option<i32> {
    let (ifinfo, attrs) = parse_ifinfomsg(payload)?;

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

    Some(ifinfo.ifi_index)
}

/// parses interface driver
pub fn parse_interface_driver(path: &Path) -> std::io::Result<Option<String>> {
    let path = path.join("device").join("driver");

    match fs::read_link(path) {
        Ok(target) => Ok(target
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(err) => Err(err),
    }
}

/// parses interface index
pub fn parse_interface_index(path: &Path) -> std::io::Result<i32> {
    let index_str = fs::read_to_string(path.join("ifindex"))?;

    index_str
        .trim()
        .parse::<i32>()
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
}

/// parses interface kind
pub fn parse_interface_kind(path: &Path) -> std::io::Result<InterfaceKind> {
    let ty = fs::read_to_string(path.join("type"))?;
    let ty = ty.trim().parse::<u32>().unwrap_or_default();

    Ok(match ty {
        ARPHRD_ETHER => InterfaceKind::Ethernet,
        ARPHRD_PPP => InterfaceKind::Ppp,
        ARPHRD_IPGRE | ARPHRD_IP6GRE | ARPHRD_SIT | ARPHRD_TUNNEL6 => InterfaceKind::Tunnel,
        ARPHRD_LOOPBACK => InterfaceKind::Loopback,
        ARPHRD_RAWIP => InterfaceKind::RawIp,
        other => InterfaceKind::Other(other),
    })
}

/// parses interface link speed (bps)
pub fn parse_interface_speed(path: &Path) -> std::io::Result<Option<u64>> {
    let path = path.join("speed");

    match fs::read_to_string(path) {
        Ok(speed) => Ok(speed
            .trim()
            .parse::<u64>()
            .ok()
            .map(|speed| speed * 1_000_000)),
        Err(_) => Ok(None),
    }
}
