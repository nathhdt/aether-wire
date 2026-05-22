#![cfg(target_os = "linux")]
//! kernel-level receive timestamps (Linux) via SO_TIMESTAMPNS

use std::mem;
use std::net::UdpSocket;
use std::os::fd::AsRawFd;

pub fn enable_so_timestampns(sock: &UdpSocket) -> bool {
    let enable: libc::c_int = 1;
    let ret = unsafe {
        libc::setsockopt(
            sock.as_raw_fd(),
            libc::SOL_SOCKET,
            libc::SO_TIMESTAMPNS,
            (&enable as *const libc::c_int).cast(),
            mem::size_of_val(&enable) as libc::socklen_t,
        )
    };
    ret == 0
}
