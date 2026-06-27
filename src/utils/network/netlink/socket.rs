//! Netlink socket module

use rustix::net::{
    AddressFamily, SocketFlags, SocketType, bind, netlink::SocketAddrNetlink, socket_with,
};

/// opens a Netlink socket
pub fn open_netlink_socket() -> std::io::Result<rustix::fd::OwnedFd> {
    let fd = socket_with(
        AddressFamily::NETLINK,
        SocketType::RAW,
        SocketFlags::empty(),
        None,
    )?;

    let addr = SocketAddrNetlink::new(0, 0);
    bind(&fd, &addr)?;

    Ok(fd)
}
