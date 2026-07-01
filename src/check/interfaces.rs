//! network interfaces check module

use anyhow::Result;

use crate::utils::network::{
    interfaces::{
        constants::{IF_OPER_DORMANT, IF_OPER_DOWN, IF_OPER_LOWERLAYERDOWN, IF_OPER_UP},
        get_all_interfaces,
        types::{Interface, InterfaceClass, InterfaceKind},
    },
    netlink::constants::{
        NETDEV_XDP_ACT_BASIC, NETDEV_XDP_ACT_RX_SG, NETDEV_XDP_ACT_XSK_ZEROCOPY, XDP_ATTACHED_DRV,
        XDP_ATTACHED_HW, XDP_ATTACHED_MULTI, XDP_ATTACHED_NONE, XDP_ATTACHED_SKB,
    },
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
            IF_OPER_DOWN => Self::Down,
            IF_OPER_LOWERLAYERDOWN => Self::LowerLayerDown,
            IF_OPER_DORMANT => Self::Dormant,
            IF_OPER_UP => Self::Up,
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

pub fn check_interfaces(iface_filter: Option<&Interface>) -> Result<Vec<InterfaceChecks>> {
    let mut interfaces_checks = Vec::new();

    let interfaces = match iface_filter {
        Some(interface) => vec![interface.clone()],
        None => get_all_interfaces()?,
    };

    for interface in interfaces {
        let type_check = Check {
            label: "type".into(),
            value: match interface.kind {
                InterfaceKind::Ethernet => "Ethernet".into(),
                InterfaceKind::Loopback => "loopback".into(),
                InterfaceKind::Ppp => "PPP".into(),
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

        let driver_check = Check {
            label: "driver".into(),
            value: match interface.kind {
                InterfaceKind::Loopback => "n/a".into(),
                _ => interface.driver.as_deref().unwrap_or("unknown").to_owned(),
            },
            status: Status::Info,
            note: None,
        };

        let operstate = interface
            .operstate
            .map(OperState::from_u8)
            .unwrap_or(OperState::Unknown);
        let xdp_basic = interface
            .xdp_features
            .map(|f| f & NETDEV_XDP_ACT_BASIC as u64 != 0);
        let xdp_zerocopy = interface
            .xdp_features
            .map(|f| f & NETDEV_XDP_ACT_XSK_ZEROCOPY as u64 != 0);
        let xdp_multi_buffer = interface
            .xdp_features
            .map(|f| f & NETDEV_XDP_ACT_RX_SG as u64 != 0);

        let netlink_checks = vec![
            Check {
                label: "state".into(),
                value: operstate.as_str().into(),
                status: operstate.status(),
                note: None,
            },
            {
                let (status, value, note) = if xdp_basic.is_none() {
                    (Status::Warn, String::new(), Some("unavailable".into()))
                } else {
                    let mut present = Vec::new();
                    let mut missing = Vec::new();

                    if xdp_basic == Some(true) {
                        present.push("basic");
                    } else {
                        missing.push("basic");
                    }
                    if xdp_zerocopy == Some(true) {
                        present.push("xsk-zc");
                    } else {
                        missing.push("xsk-zc");
                    }
                    if xdp_multi_buffer == Some(true) {
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
                value: match interface.xdp_attached {
                    None | Some(XDP_ATTACHED_NONE) => "none".into(),
                    Some(XDP_ATTACHED_DRV) => match interface.xdp_prog_id {
                        Some(id) => format!("drv (prog #{id})"),
                        None => "drv".into(),
                    },
                    Some(XDP_ATTACHED_SKB) => match interface.xdp_prog_id {
                        Some(id) => format!("skb (prog #{id})"),
                        None => "skb".into(),
                    },
                    Some(XDP_ATTACHED_HW) => match interface.xdp_prog_id {
                        Some(id) => format!("hw (prog #{id})"),
                        None => "hw".into(),
                    },
                    Some(XDP_ATTACHED_MULTI) => "multi".into(),
                    Some(_) => "unknown".into(),
                },
                status: match interface.xdp_attached {
                    Some(XDP_ATTACHED_DRV) | Some(XDP_ATTACHED_HW) => Status::Ok,
                    Some(XDP_ATTACHED_SKB) => Status::Warn,
                    _ => Status::Info,
                },
                note: match interface.xdp_attached {
                    Some(XDP_ATTACHED_SKB) => Some("generic mode".into()),
                    _ => None,
                },
            },
            Check {
                label: "MTU".into(),
                value: interface
                    .mtu
                    .map(|v| v.to_string())
                    .unwrap_or_else(|| "unknown".into()),
                status: Status::Info,
                note: None,
            },
            Check {
                label: "queues".into(),
                value: match (interface.rx_queues, interface.tx_queues) {
                    (Some(rx), Some(tx)) => format!("{rx} rx / {tx} tx"),
                    (Some(rx), None) => format!("{rx} rx"),
                    (None, Some(tx)) => format!("{tx} tx"),
                    (None, None) => "unknown".into(),
                },
                status: Status::Info,
                note: None,
            },
        ];

        let mut checks = vec![type_check, class_check, driver_check];
        checks.extend(netlink_checks);

        interfaces_checks.push(InterfaceChecks {
            interface: interface.name,
            checks,
        });
    }

    Ok(interfaces_checks)
}
