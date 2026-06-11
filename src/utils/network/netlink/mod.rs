//! Netlink utilities module

pub mod builder;
pub mod constants;
pub mod parser;
pub mod socket;
pub mod types;

use rustix::net::{RecvFlags, SendFlags, recv, send};

use parser::{parse_nlmsg_error, recv_is_done};
use socket::open_netlink_socket;

/// sends a netlink request and returns the full response buffer
pub fn request(req: &[u8]) -> std::io::Result<Vec<u8>> {
    let fd = open_netlink_socket()?;

    send(&fd, req, SendFlags::empty())
        .map_err(|e| std::io::Error::from_raw_os_error(e.raw_os_error()))?;

    let mut result = Vec::new();
    let mut buf = vec![0u8; 8192];

    loop {
        let (len, _) = recv(&fd, &mut buf, RecvFlags::empty())
            .map_err(|e| std::io::Error::from_raw_os_error(e.raw_os_error()))?;

        let chunk = &buf[..len];

        if let Some(errno) = parse_nlmsg_error(chunk) {
            return Err(std::io::Error::from_raw_os_error(-errno));
        }

        result.extend_from_slice(chunk);

        if recv_is_done(chunk) {
            break;
        }
    }

    Ok(result)
}
