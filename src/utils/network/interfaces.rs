//! network interfaces utilities module

use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::{fmt, fs, path::Path};

use super::constants::{
    AF_INET, AF_INET6, ARPHRD_ETHER, ARPHRD_IP6GRE, ARPHRD_IPGRE, ARPHRD_LOOPBACK, ARPHRD_PPP,
    ARPHRD_RAWIP, ARPHRD_SIT, ARPHRD_TUNNEL6,
};
use super::netlink::{
    builder::{build_getaddr_dump_request, build_getlink_dump_request, build_getlink_request},
    constants::{
        IFA_LOCAL, IFLA_MTU, IFLA_NUM_RX_QUEUES, IFLA_NUM_TX_QUEUES, IFLA_OPERSTATE, IFLA_XDP,
        IFLA_XDP_ATTACHED, IFLA_XDP_FEATURES, IFLA_XDP_PROG_ID, RTM_NEWADDR, RTM_NEWLINK,
    },
    parser::{NlMsgIter, RtAttrIter, parse_ifinfomsg},
    request,
    types::IfAddrMsg,
};

#[derive(Debug)]
pub enum InterfaceError {
    InterfaceNotFound,
    Io(std::io::Error),
}

impl fmt::Display for InterfaceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InterfaceNotFound => write!(f, "interface not found"),
            Self::Io(err) => write!(f, "{err}"),
        }
    }
}

impl std::error::Error for InterfaceError {}

impl From<std::io::Error> for InterfaceError {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Interface {
    pub index: i32,
    pub name: String,
    pub kind: InterfaceKind,
    pub class: InterfaceClass,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InterfaceKind {
    Ethernet,
    Loopback,
    Ppp,
    Tunnel,
    RawIp,
    Other(u32),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InterfaceClass {
    Device,
    Virtual,
}

/// interface details from Netlink RTM_GETLINK
pub struct InterfaceDetails {
    pub mtu: Option<u32>,
    pub operstate: Option<u8>,
    pub rx_queues: Option<u32>,
    pub tx_queues: Option<u32>,
    pub xdp_features: Option<u64>,
    pub xdp_attached: Option<u8>,
    pub xdp_prog_id: Option<u32>,
}

/// interface address from Netlink RTM_GETADDR
pub struct InterfaceAddress {
    pub addr: IpAddr,
    #[allow(unused)]
    pub prefix_len: u8,
}

fn get_interface_index(path: &Path) -> std::io::Result<i32> {
    let index_str = fs::read_to_string(path.join("ifindex"))?;

    index_str
        .trim()
        .parse::<i32>()
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
}

fn get_interface_kind(path: &Path) -> std::io::Result<InterfaceKind> {
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

fn get_interface_class(path: &Path) -> InterfaceClass {
    if path.join("device").exists() {
        InterfaceClass::Device
    } else {
        InterfaceClass::Virtual
    }
}

/// parses a RTM_NEWLINK payload into an interface index and details
fn parse_interface_details(payload: &[u8]) -> Option<(i32, InterfaceDetails)> {
    let (ifinfo, attrs) = parse_ifinfomsg(payload)?;

    let mut mtu = None;
    let mut operstate = None;
    let mut rx_queues = None;
    let mut tx_queues = None;
    let mut xdp_features = None;
    let mut xdp_attached = None;
    let mut xdp_prog_id = None;

    for (attr_type, attr_data) in RtAttrIter::new(attrs) {
        match attr_type {
            t if t == IFLA_OPERSTATE && !attr_data.is_empty() => {
                operstate = Some(attr_data[0]);
            }
            t if t == IFLA_XDP => {
                for (xdp_type, xdp_data) in RtAttrIter::new(attr_data) {
                    if xdp_type == IFLA_XDP_FEATURES
                        && xdp_data.len() >= 8
                        && let Ok(bytes) = xdp_data[..8].try_into()
                    {
                        xdp_features = Some(u64::from_ne_bytes(bytes));
                    }
                    if xdp_type == IFLA_XDP_ATTACHED && !xdp_data.is_empty() {
                        xdp_attached = Some(xdp_data[0]);
                    }
                    if xdp_type == IFLA_XDP_PROG_ID
                        && xdp_data.len() >= 4
                        && let Ok(bytes) = xdp_data[..4].try_into()
                    {
                        xdp_prog_id = Some(u32::from_ne_bytes(bytes));
                    }
                }
            }
            t if t == IFLA_MTU && attr_data.len() >= 4 => {
                if let Ok(bytes) = attr_data[..4].try_into() {
                    mtu = Some(u32::from_ne_bytes(bytes));
                }
            }
            t if t == IFLA_NUM_RX_QUEUES && attr_data.len() >= 4 => {
                if let Ok(bytes) = attr_data[..4].try_into() {
                    rx_queues = Some(u32::from_ne_bytes(bytes));
                }
            }
            t if t == IFLA_NUM_TX_QUEUES && attr_data.len() >= 4 => {
                if let Ok(bytes) = attr_data[..4].try_into() {
                    tx_queues = Some(u32::from_ne_bytes(bytes));
                }
            }
            _ => {}
        }
    }

    Some((
        ifinfo.ifi_index,
        InterfaceDetails {
            mtu,
            operstate,
            rx_queues,
            tx_queues,
            xdp_features,
            xdp_attached,
            xdp_prog_id,
        },
    ))
}

/// parses a RTM_NEWADDR payload into an interface address
fn parse_interface_address(payload: &[u8]) -> Option<(i32, InterfaceAddress)> {
    if payload.len() < IfAddrMsg::SIZE {
        return None;
    }

    let msg: IfAddrMsg = unsafe { core::ptr::read_unaligned(payload.as_ptr() as *const IfAddrMsg) };

    let attrs = &payload[IfAddrMsg::SIZE..];

    for (attr_type, attr_data) in RtAttrIter::new(attrs) {
        if attr_type != IFA_LOCAL {
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

        return Some((
            msg.ifa_index as i32,
            InterfaceAddress {
                addr,
                prefix_len: msg.ifa_prefixlen,
            },
        ));
    }

    None
}

/// returns the driver associated with a network interface
pub fn get_interface_driver(name: &str) -> Result<Option<String>, InterfaceError> {
    let path = Path::new("/sys/class/net")
        .join(name)
        .join("device")
        .join("driver");

    match fs::read_link(path) {
        Ok(target) => Ok(target
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())),

        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
            let interface_path = Path::new("/sys/class/net").join(name);

            if !interface_path.exists() {
                return Err(InterfaceError::InterfaceNotFound);
            }

            Ok(None)
        }

        Err(err) => Err(err.into()),
    }
}

/// returns all network interfaces
pub fn get_interfaces() -> std::io::Result<Vec<Interface>> {
    let mut interfaces = Vec::new();

    for entry in fs::read_dir("/sys/class/net")? {
        let entry = entry?;

        let name = entry.file_name().to_string_lossy().into_owned();
        let path = entry.path();

        let index = get_interface_index(&path)?;
        let kind = get_interface_kind(&path)?;
        let class = get_interface_class(&path);

        interfaces.push(Interface {
            index,
            name,
            kind,
            class,
        });
    }

    interfaces.sort_unstable_by(|a, b| a.name.cmp(&b.name));

    Ok(interfaces)
}

/// returns a network interface
pub fn get_interface(name: &str) -> Result<Interface, InterfaceError> {
    let path = Path::new("/sys/class/net").join(name);

    if !path.exists() {
        return Err(InterfaceError::InterfaceNotFound);
    }

    let index = get_interface_index(&path)?;
    let kind = get_interface_kind(&path)?;
    let class = get_interface_class(&path);

    Ok(Interface {
        index,
        name: name.to_owned(),
        kind,
        class,
    })
}

/// returns whether a network interface exists
pub fn interface_exists(name: &str) -> bool {
    Path::new("/sys/class/net").join(name).exists()
}

/// queries interface details via Netlink RTM_GETLINK
pub fn get_interface_details(ifindex: i32) -> Result<Option<InterfaceDetails>, InterfaceError> {
    let response = request(&build_getlink_request(ifindex, 1337))?;

    for (msg_type, payload) in NlMsgIter::new(&response) {
        if msg_type != RTM_NEWLINK {
            continue;
        }

        if let Some((_, details)) = parse_interface_details(payload) {
            return Ok(Some(details));
        }
    }

    Ok(None)
}

/// queries all interface details via a Netlink RTM_GETLINK dump
pub fn get_all_interface_details() -> Result<HashMap<i32, InterfaceDetails>, InterfaceError> {
    let response = request(&build_getlink_dump_request(1337))?;
    let mut map = HashMap::new();

    for (msg_type, payload) in NlMsgIter::new(&response) {
        if msg_type != RTM_NEWLINK {
            continue;
        }

        if let Some((ifindex, details)) = parse_interface_details(payload) {
            map.insert(ifindex, details);
        }
    }

    Ok(map)
}

/// queries addresses for a specific interface via Netlink RTM_GETADDR
pub fn get_interface_addresses(ifindex: i32) -> Result<Vec<InterfaceAddress>, InterfaceError> {
    let response = request(&build_getaddr_dump_request(1337))?;
    let mut addresses = Vec::new();

    for (msg_type, payload) in NlMsgIter::new(&response) {
        if msg_type != RTM_NEWADDR {
            continue;
        }

        if let Some((idx, addr)) = parse_interface_address(payload)
            && idx == ifindex
        {
            addresses.push(addr);
        }
    }

    Ok(addresses)
}
