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
    netlink::{
        extract_netlink_ifindex, parse_netlink_interface_address, parse_netlink_interface_details,
    },
    sysfs::parse_sysfs_interface,
};
use types::Interface;

/// chosen Netlink seqnum
const NETLINK_SEQ: u32 = 1;

/// returns a network interface
pub fn get_interface(name: &str) -> Result<Interface, InterfaceError> {
    let path = Path::new("/sys/class/net").join(name);

    if !path.exists() {
        return Err(InterfaceError::InterfaceNotFound);
    }

    let mut interface = Interface::new(name.to_owned());

    // sysfs
    parse_sysfs_interface(&path, &mut interface)?;

    // netlink (RTM_GETLINK)
    let response = request(&build_getlink_request(interface.index, NETLINK_SEQ))?;

    for (msg_type, payload) in NlMsgIter::new(&response) {
        if msg_type != RTM_NEWLINK {
            continue;
        }

        if let Some(idx) = extract_netlink_ifindex(payload)
            && idx == interface.index
        {
            parse_netlink_interface_details(payload, &mut interface);
            break;
        }
    }

    // netlink (RTM_GETADDR)
    let response = request(&build_getaddr_dump_request(NETLINK_SEQ))?;

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

        let mut interface = Interface::new(name);

        // sysfs
        parse_sysfs_interface(&path, &mut interface)?;

        interfaces.push(interface);
    }

    let index_map: HashMap<i32, usize> = interfaces
        .iter()
        .enumerate()
        .map(|(i, iface)| (iface.index, i))
        .collect();

    // netlink (RTM_GETLINK)
    let response = request(&build_getlink_dump_request(NETLINK_SEQ))?;

    for (msg_type, payload) in NlMsgIter::new(&response) {
        if msg_type != RTM_NEWLINK {
            continue;
        }

        if let Some(idx) = extract_netlink_ifindex(payload)
            && let Some(&i) = index_map.get(&idx)
        {
            parse_netlink_interface_details(payload, &mut interfaces[i]);
        }
    }

    // netlink (RTM_GETADDR)
    let response = request(&build_getaddr_dump_request(NETLINK_SEQ))?;

    for (msg_type, payload) in NlMsgIter::new(&response) {
        if msg_type != RTM_NEWADDR {
            continue;
        }

        if let Some(idx) = extract_netlink_ifindex(payload)
            && let Some(&i) = index_map.get(&idx)
        {
            parse_netlink_interface_address(payload, &mut interfaces[i]);
        }
    }

    interfaces.sort_unstable_by(|a, b| a.name.cmp(&b.name));

    Ok(interfaces)
}
