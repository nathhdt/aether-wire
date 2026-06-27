//! network address resolution utilities module

use std::fmt;
use std::net::{IpAddr, ToSocketAddrs};

#[derive(Debug)]
pub enum ResolveError {
    InvalidAddress(String),
    NotFound(String),
}

impl fmt::Display for ResolveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidAddress(s) => write!(f, "invalid address '{s}'"),
            Self::NotFound(s) => write!(f, "could not resolve '{s}'"),
        }
    }
}

impl std::error::Error for ResolveError {}

/// validates a hostname (RFC 1123)
fn is_valid_hostname(hostname: &str) -> bool {
    let hostname = hostname.strip_suffix('.').unwrap_or(hostname);
    if hostname.is_empty() || hostname.len() > 253 {
        return false;
    }
    hostname.split('.').all(|label| {
        !label.is_empty()
            && label.len() <= 63
            && !label.starts_with('-')
            && !label.ends_with('-')
            && label.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
    })
}

/// resolves a host identifier (IPv4, IPv6 or hostname)
pub fn resolve(server: &str) -> Result<Vec<IpAddr>, ResolveError> {
    if let Ok(ip) = server.parse::<IpAddr>() {
        return Ok(vec![ip]);
    }

    if !is_valid_hostname(server) {
        return Err(ResolveError::InvalidAddress(server.to_owned()));
    }

    let addrs: Vec<IpAddr> = (server, 0u16)
        .to_socket_addrs()
        .map_err(|_| ResolveError::NotFound(server.to_owned()))?
        .map(|addr| addr.ip())
        .collect();

    if addrs.is_empty() {
        Err(ResolveError::NotFound(server.to_owned()))
    } else {
        Ok(addrs)
    }
}
