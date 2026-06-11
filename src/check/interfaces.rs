//! network interfaces check module

use anyhow::Result;
use std::collections::HashMap;

use crate::utils::network::interfaces::{
    InterfaceClass, InterfaceError, InterfaceKind, get_interface_driver, get_interfaces,
};
use crate::utils::network::netlink::{
    builder::build_getlink_dump_request,
    constants::{
        IFLA_MTU, IFLA_NUM_RX_QUEUES, IFLA_NUM_TX_QUEUES, IFLA_OPERSTATE, IFLA_XDP,
        IFLA_XDP_ATTACHED, IFLA_XDP_FEATURES, IFLA_XDP_PROG_ID, NETDEV_XDP_ACT_BASIC,
        NETDEV_XDP_ACT_RX_SG, NETDEV_XDP_ACT_XSK_ZEROCOPY, RTM_NEWLINK, XDP_ATTACHED_DRV,
        XDP_ATTACHED_HW, XDP_ATTACHED_MULTI, XDP_ATTACHED_NONE, XDP_ATTACHED_SKB,
    },
    parser::{NlMsgIter, RtAttrIter, parse_ifinfomsg},
    request,
};

use super::{Check, InterfaceChecks, Status};

/// interface operational state
enum OperState {
    Up,
    Down,
    LowerLayerDown,
    Dormant,
    Unknown,
}

impl OperState {
    fn from_u8(value: u8) -> Self {
        match value {
            2 => Self::Down,
            3 => Self::LowerLayerDown,
            5 => Self::Dormant,
            6 => Self::Up,
            _ => Self::Unknown,
        }
    }

    fn as_str(&self) -> &'static str {
        match self {
            Self::Up => "up",
            Self::Down => "down",
            Self::LowerLayerDown => "lower layer down",
            Self::Dormant => "dormant",
            Self::Unknown => "unknown",
        }
    }

    fn status(&self) -> Status {
        match self {
            Self::Up => Status::Ok,
            Self::Down | Self::LowerLayerDown => Status::Warn,
            _ => Status::Info,
        }
    }
}

/// interface informations
struct InterfaceInfo {
    operstate: OperState,
    xdp_basic: Option<bool>,
    xdp_zerocopy: Option<bool>,
    xdp_multi_buffer: Option<bool>,
    xdp_attached: Option<u8>,
    xdp_prog_id: Option<u32>,
    mtu: Option<u32>,
    rx_queues: Option<u32>,
    tx_queues: Option<u32>,
}

/// queries all interfaces via a single RTM_GETLINK dump
fn dump_interfaces() -> Result<HashMap<i32, InterfaceInfo>, std::io::Error> {
    let response = request(&build_getlink_dump_request(1337))?;
    let mut map = HashMap::new();

    for (msg_type, payload) in NlMsgIter::new(&response) {
        if msg_type != RTM_NEWLINK {
            continue;
        }

        let (ifinfo, attrs) = match parse_ifinfomsg(payload) {
            Some(r) => r,
            None => continue,
        };

        let mut operstate = None;
        let mut xdp_features = None;
        let mut xdp_attached = None;
        let mut xdp_prog_id = None;
        let mut mtu = None;
        let mut rx_queues = None;
        let mut tx_queues = None;

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

        map.insert(
            ifinfo.ifi_index,
            InterfaceInfo {
                operstate: operstate
                    .map(OperState::from_u8)
                    .unwrap_or(OperState::Unknown),
                xdp_basic: xdp_features.map(|f| f & NETDEV_XDP_ACT_BASIC as u64 != 0),
                xdp_zerocopy: xdp_features.map(|f| f & NETDEV_XDP_ACT_XSK_ZEROCOPY as u64 != 0),
                xdp_multi_buffer: xdp_features.map(|f| f & NETDEV_XDP_ACT_RX_SG as u64 != 0),
                xdp_attached,
                xdp_prog_id,
                mtu,
                rx_queues,
                tx_queues,
            },
        );
    }

    Ok(map)
}

pub fn check_interfaces() -> Result<Vec<InterfaceChecks>> {
    let mut interfaces_checks = Vec::new();

    let interfaces = get_interfaces()?;
    let netlink_data = dump_interfaces();

    for interface in interfaces {
        let type_check = Check {
            label: "type".into(),
            value: match interface.kind {
                InterfaceKind::Ethernet => "ethernet".into(),
                InterfaceKind::Loopback => "loopback".into(),
                InterfaceKind::Ppp => "ppp".into(),
                InterfaceKind::Tunnel => "tunnel".into(),
                InterfaceKind::RawIp => "none (raw IP)".into(),
                InterfaceKind::Other(kind) => format!("other ({kind})"),
            },
            status: Status::Info,
            note: None,
        };

        let class_check = Check {
            label: "class".into(),
            value: match interface.class {
                InterfaceClass::Device => "device".into(),
                InterfaceClass::Virtual => "virtual".into(),
            },
            status: Status::Info,
            note: None,
        };

        let driver_check = match interface.kind {
            InterfaceKind::Loopback => Check {
                label: "driver".into(),
                value: "n/a".into(),
                status: Status::Info,
                note: None,
            },

            _ => match get_interface_driver(&interface.name) {
                Ok(Some(driver)) => Check {
                    label: "driver".into(),
                    value: driver,
                    status: Status::Info,
                    note: None,
                },

                Ok(None) => Check {
                    label: "driver".into(),
                    value: "unknown".into(),
                    status: Status::Info,
                    note: None,
                },

                Err(InterfaceError::Io(err)) => Check {
                    label: "driver".into(),
                    value: "unknown".into(),
                    status: Status::Warn,
                    note: Some(format!("unable to determine driver: {err}")),
                },

                Err(_) => Check {
                    label: "driver".into(),
                    value: "unknown".into(),
                    status: Status::Warn,
                    note: Some("unable to determine driver".into()),
                },
            },
        };

        let netlink_checks = match netlink_data.as_ref().map(|m| m.get(&interface.index)) {
            Err(err) => vec![Check {
                label: "netlink".into(),
                value: "error".into(),
                status: Status::Warn,
                note: Some(err.to_string()),
            }],

            Ok(None) => vec![Check {
                label: "netlink".into(),
                value: "no response".into(),
                status: Status::Warn,
                note: None,
            }],

            Ok(Some(info)) => vec![
                Check {
                    label: "state".into(),
                    value: info.operstate.as_str().into(),
                    status: info.operstate.status(),
                    note: None,
                },
                {
                    let (status, value, note) = if info.xdp_basic.is_none() {
                        (Status::Warn, String::new(), Some("unavailable".into()))
                    } else {
                        let mut present = Vec::new();
                        let mut missing = Vec::new();

                        if info.xdp_basic == Some(true) {
                            present.push("basic");
                        } else {
                            missing.push("basic");
                        }
                        if info.xdp_zerocopy == Some(true) {
                            present.push("xsk-zc");
                        } else {
                            missing.push("xsk-zc");
                        }
                        if info.xdp_multi_buffer == Some(true) {
                            present.push("rx-sg");
                        } else {
                            missing.push("rx-sg");
                        }

                        if missing.is_empty() {
                            (Status::Ok, present.join(","), None)
                        } else if present.is_empty() {
                            (Status::Warn, String::new(), Some("none advertised".into()))
                        } else {
                            (
                                Status::Warn,
                                present.join(","),
                                Some(format!("{} not advertised", missing.join(","))),
                            )
                        }
                    };

                    Check {
                        label: "XDP features".into(),
                        value,
                        status,
                        note,
                    }
                },
                Check {
                    label: "XDP attached".into(),
                    value: match info.xdp_attached {
                        None | Some(XDP_ATTACHED_NONE) => "none".into(),
                        Some(XDP_ATTACHED_DRV) => match info.xdp_prog_id {
                            Some(id) => format!("drv (prog #{id})"),
                            None => "drv".into(),
                        },
                        Some(XDP_ATTACHED_SKB) => match info.xdp_prog_id {
                            Some(id) => format!("skb (prog #{id})"),
                            None => "skb".into(),
                        },
                        Some(XDP_ATTACHED_HW) => match info.xdp_prog_id {
                            Some(id) => format!("hw (prog #{id})"),
                            None => "hw".into(),
                        },
                        Some(XDP_ATTACHED_MULTI) => "multi".into(),
                        Some(_) => "unknown".into(),
                    },
                    status: match info.xdp_attached {
                        Some(XDP_ATTACHED_DRV) | Some(XDP_ATTACHED_HW) => Status::Ok,
                        Some(XDP_ATTACHED_SKB) => Status::Warn,
                        _ => Status::Info,
                    },
                    note: match info.xdp_attached {
                        Some(XDP_ATTACHED_SKB) => Some("generic mode".into()),
                        _ => None,
                    },
                },
                Check {
                    label: "MTU".into(),
                    value: info
                        .mtu
                        .map(|v| v.to_string())
                        .unwrap_or_else(|| "unknown".into()),
                    status: Status::Info,
                    note: None,
                },
                Check {
                    label: "queues".into(),
                    value: match (info.rx_queues, info.tx_queues) {
                        (Some(rx), Some(tx)) => format!("{rx} rx / {tx} tx"),
                        (Some(rx), None) => format!("{rx} rx"),
                        (None, Some(tx)) => format!("{tx} tx"),
                        (None, None) => "unknown".into(),
                    },
                    status: match info.rx_queues {
                        Some(n) if n > 1 => Status::Ok,
                        _ => Status::Info,
                    },
                    note: None,
                },
            ],
        };

        let mut checks = vec![type_check, class_check, driver_check];
        checks.extend(netlink_checks);

        interfaces_checks.push(InterfaceChecks {
            interface: interface.name,
            checks,
        });
    }

    Ok(interfaces_checks)
}
