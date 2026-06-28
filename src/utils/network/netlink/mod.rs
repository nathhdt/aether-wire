//! Netlink utilities module

pub mod builder;
pub mod constants;
pub mod parser;
pub mod socket;
pub mod types;

use std::io::IoSliceMut;

use rustix::net::{RecvAncillaryBuffer, RecvFlags, ReturnFlags, SendFlags, recvmsg, send};

use parser::{parse_nlmsg_error, recv_is_done};
use socket::open_netlink_socket;

/// sends a Netlink request and returns the full response buffer
pub fn request(req: &[u8]) -> std::io::Result<Vec<u8>> {
    let fd = open_netlink_socket()?;

    send(&fd, req, SendFlags::empty())?;

    let mut result = Vec::new();
    let mut buf = vec![0u8; 64 * 1024];
    let mut iov = [IoSliceMut::new(&mut buf)];
    let mut control = RecvAncillaryBuffer::default();

    loop {
        let msg = recvmsg(&fd, &mut iov, &mut control, RecvFlags::TRUNC)?;

        if msg.flags.contains(ReturnFlags::TRUNC) {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "truncated Netlink datagram",
            ));
        }

        let chunk = &iov[0][..msg.bytes];

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
