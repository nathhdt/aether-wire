//! network interfaces utilities module

pub mod constants;
pub mod error;
pub mod parser;
pub mod types;

use std::collections::HashMap;
use std::fs;
use std::path::Path;

use super::netlink::{
    builder::{build_getaddr_dump_request, build_getlink_dump_request, build_getlink_request},
    constants::{RTM_NEWADDR, RTM_NEWLINK},
    parser::NlMsgIter,
    request,
};

use error::InterfaceError;
use parser::{
    parse_interface_class, parse_interface_driver, parse_interface_index, parse_interface_kind,
    parse_netlink_ifindex, parse_netlink_interface_address, parse_netlink_interface_details,
};
use types::Interface;

/// returns a network interface
pub fn get_interface(name: &str) -> Result<Interface, InterfaceError> {
    let path = Path::new("/sys/class/net").join(name);

    if !path.exists() {
        return Err(InterfaceError::InterfaceNotFound);
    }

    // sysfs
    let index = parse_interface_index(&path)?;
    let kind = parse_interface_kind(&path)?;
    let class = parse_interface_class(&path);
    let driver = parse_interface_driver(&path)?;

    let mut interface = Interface {
        index,
        name: name.to_owned(),
        kind,
        class,
        driver,

        mtu: None,
        operstate: None,
        rx_queues: None,
        tx_queues: None,
        xdp_features: None,
        xdp_attached: None,
        xdp_prog_id: None,

        addresses: Vec::new(),
    };

    // netlink (RTM_GETLINK)
    let response = request(&build_getlink_request(interface.index, 1337))?;

    for (msg_type, payload) in NlMsgIter::new(&response) {
        if msg_type != RTM_NEWLINK {
            continue;
        }

        if let Some(idx) = parse_netlink_ifindex(payload)
            && idx == interface.index
        {
            parse_netlink_interface_details(payload, &mut interface);
            break;
        }
    }

    // netlink (RTM_GETADDR)
    let response = request(&build_getaddr_dump_request(1337))?;

    for (msg_type, payload) in NlMsgIter::new(&response) {
        if msg_type != RTM_NEWADDR {
            continue;
        }

        parse_netlink_interface_address(payload, &mut interface);
    }

    Ok(interface)
}

/// returns all network interfaces
pub fn get_all_interfaces() -> Result<Vec<Interface>, InterfaceError> {
    let mut interfaces = Vec::new();

    for entry in fs::read_dir("/sys/class/net")? {
        let entry = entry?;

        let name = entry.file_name().to_string_lossy().into_owned();
        let path = entry.path();

        // sysfs
        let index = parse_interface_index(&path)?;
        let kind = parse_interface_kind(&path)?;
        let class = parse_interface_class(&path);
        let driver = parse_interface_driver(&path)?;

        interfaces.push(Interface {
            index,
            name,
            kind,
            class,
            driver,

            mtu: None,
            operstate: None,
            rx_queues: None,
            tx_queues: None,
            xdp_features: None,
            xdp_attached: None,
            xdp_prog_id: None,

            addresses: Vec::new(),
        });
    }

    let index_map: HashMap<i32, usize> = interfaces
        .iter()
        .enumerate()
        .map(|(i, iface)| (iface.index, i))
        .collect();

    // netlink (RTM_GETLINK)
    let response = request(&build_getlink_dump_request(1337))?;

    for (msg_type, payload) in NlMsgIter::new(&response) {
        if msg_type != RTM_NEWLINK {
            continue;
        }

        if let Some(idx) = parse_netlink_ifindex(payload)
            && let Some(&i) = index_map.get(&idx)
        {
            parse_netlink_interface_details(payload, &mut interfaces[i]);
        }
    }

    // netlink (RTM_GETADDR)
    let response = request(&build_getaddr_dump_request(1337))?;

    for (msg_type, payload) in NlMsgIter::new(&response) {
        if msg_type != RTM_NEWADDR {
            continue;
        }

        if let Some(idx) = parse_netlink_ifindex(payload)
            && let Some(&i) = index_map.get(&idx)
        {
            parse_netlink_interface_address(payload, &mut interfaces[i]);
        }
    }

    interfaces.sort_unstable_by(|a, b| a.name.cmp(&b.name));

    Ok(interfaces)
}
