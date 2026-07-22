//! network interfaces error utilities module

use std::fmt;
use std::net::IpAddr;

#[derive(Debug)]
pub enum InterfaceError {
    NotFound(String),
    NotUp(String),
    NoAddresses(String),
    AddrNotConfigured { iface: String, addr: IpAddr },
    Io(std::io::Error),
}

impl fmt::Display for InterfaceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotFound(name) => write!(f, "interface '{name}' not found"),
            Self::NotUp(name) => write!(f, "interface '{name}' is not up"),
            Self::NoAddresses(name) => write!(f, "interface '{name}' has no configured addresses"),
            Self::AddrNotConfigured { iface, addr } => {
                write!(
                    f,
                    "address '{addr}' is not configured on interface '{iface}'"
                )
            }
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
