//! network interfaces sysfs parsing utilities module

use std::{fs, path::Path};

use super::super::{
    constants::{
        ARPHRD_ETHER, ARPHRD_IP6GRE, ARPHRD_IPGRE, ARPHRD_LOOPBACK, ARPHRD_PPP, ARPHRD_RAWIP,
        ARPHRD_SIT, ARPHRD_TUNNEL6,
    },
    types::{Interface, InterfaceClass, InterfaceKind},
};

/// parses interface class
fn parse_interface_class(path: &Path) -> InterfaceClass {
    if path.join("device").exists() {
        InterfaceClass::Device
    } else {
        InterfaceClass::Virtual
    }
}

/// parses interface driver
fn parse_interface_driver(path: &Path) -> std::io::Result<Option<String>> {
    let path = path.join("device").join("driver");

    match fs::read_link(path) {
        Ok(target) => Ok(target
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())),
        Err(_) => Ok(None),
    }
}

/// parses interface index
fn parse_interface_index(path: &Path) -> std::io::Result<i32> {
    let index_str = fs::read_to_string(path.join("ifindex"))?;

    index_str
        .trim()
        .parse::<i32>()
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
}

/// parses interface kind
fn parse_interface_kind(path: &Path) -> InterfaceKind {
    let ty = match fs::read_to_string(path.join("type")) {
        Ok(ty) => ty.trim().parse::<u32>().unwrap_or_default(),
        Err(_) => 0,
    };

    match ty {
        ARPHRD_ETHER => InterfaceKind::Ethernet,
        ARPHRD_PPP => InterfaceKind::Ppp,
        ARPHRD_IPGRE | ARPHRD_IP6GRE | ARPHRD_SIT | ARPHRD_TUNNEL6 => InterfaceKind::Tunnel,
        ARPHRD_LOOPBACK => InterfaceKind::Loopback,
        ARPHRD_RAWIP => InterfaceKind::RawIp,
        other => InterfaceKind::Other(other),
    }
}

/// parses interface link speed (bps)
fn parse_interface_speed(path: &Path) -> std::io::Result<Option<u64>> {
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

/// parses sysfs data into an Interface
pub fn parse_sysfs_interface(path: &Path, interface: &mut Interface) -> std::io::Result<()> {
    interface.index = parse_interface_index(path)?;
    interface.kind = parse_interface_kind(path);
    interface.class = parse_interface_class(path);
    interface.driver = parse_interface_driver(path)?;
    interface.speed = parse_interface_speed(path)?;

    Ok(())
}
