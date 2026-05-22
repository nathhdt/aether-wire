#![cfg(target_os = "macos")]
//! kernel-level receive timestamps (macOS) via SO_TIMESTAMP

use std::mem;
use std::net::UdpSocket;
use std::os::fd::AsRawFd;

pub fn enable_so_timestamp(sock: &UdpSocket) -> bool {
    let enable: libc::c_int = 1;
    let ret = unsafe {
        libc::setsockopt(
            sock.as_raw_fd(),
            libc::SOL_SOCKET,
            libc::SO_TIMESTAMP,
            (&enable as *const libc::c_int).cast(),
            mem::size_of_val(&enable) as libc::socklen_t,
        )
    };
    ret == 0
}
