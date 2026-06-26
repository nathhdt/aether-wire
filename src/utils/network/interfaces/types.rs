//! network interfaces types utilities module

use std::net::IpAddr;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Interface {
    // sysfs
    pub index: i32,
    pub name: String,
    pub kind: InterfaceKind,
    pub class: InterfaceClass,
    pub driver: Option<String>,

    // netlink (RTM_GETLINK)
    pub mtu: Option<u32>,
    pub operstate: Option<u8>,
    pub rx_queues: Option<u32>,
    pub tx_queues: Option<u32>,
    pub xdp_features: Option<u64>,
    pub xdp_attached: Option<u8>,
    pub xdp_prog_id: Option<u32>,

    // netlink (RTM_GETADDR)
    pub addresses: Vec<InterfaceAddress>,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InterfaceAddress {
    pub addr: IpAddr,
    #[allow(unused)]
    pub prefix_len: u8,
}
