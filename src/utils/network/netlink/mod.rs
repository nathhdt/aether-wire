//! Netlink utilities module

use rustix::net::{
    AddressFamily, SocketFlags, SocketType, bind, netlink::SocketAddrNetlink, socket_with,
};

#[allow(dead_code)]
fn open_netlink_socket() -> std::io::Result<rustix::fd::OwnedFd> {
    // socket
    let fd = socket_with(
        AddressFamily::NETLINK,
        SocketType::RAW,
        SocketFlags::empty(),
        None,
    )
    .map_err(|e| std::io::Error::from_raw_os_error(e.raw_os_error()))?;

    // addr
    let addr = SocketAddrNetlink::new(0, 0);

    // bind
    bind(&fd, &addr)?;

    Ok(fd)
}
