//! socket receive buffer configuration (cross-platform) via SO_RCVBUF

use std::mem;
use std::net::UdpSocket;

#[cfg(unix)]
use std::os::fd::AsRawFd;
#[cfg(windows)]
use std::os::windows::io::AsRawSocket;

/// clamps the target size to platform limits and applies it
pub fn set_so_rcvbuf(sock: &UdpSocket, target_bytes: usize) -> usize {
    #[cfg(unix)]
    let fd = sock.as_raw_fd() as _;
    #[cfg(windows)]
    let fd = sock.as_raw_socket() as _;

    let max_bytes = get_platform_max_rcvbuf().unwrap_or(target_bytes);
    let desired = target_bytes.min(max_bytes);

    #[cfg(target_os = "linux")]
    let desired = desired / 2;

    let desired_ffi = desired as libc::c_int;

    unsafe {
        libc::setsockopt(
            fd,
            libc::SOL_SOCKET,
            libc::SO_RCVBUF,
            (&desired_ffi as *const libc::c_int).cast(),
            mem::size_of_val(&desired_ffi) as libc::socklen_t,
        );
    }

    #[cfg(target_os = "linux")]
    {
        desired * 2
    }
    #[cfg(not(target_os = "linux"))]
    {
        desired
    }
}

#[cfg(target_os = "linux")]
fn get_platform_max_rcvbuf() -> Option<usize> {
    std::fs::read_to_string("/proc/sys/net/core/rmem_max")
        .ok()
        .and_then(|s| s.trim().parse().ok())
}

#[cfg(target_os = "macos")]
fn get_platform_max_rcvbuf() -> Option<usize> {
    let mut maxbuf: libc::c_int = 0;
    let mut len = mem::size_of_val(&maxbuf) as libc::size_t;
    let mib = [libc::CTL_KERN, libc::KERN_IPC, libc::KIPC_MAXSOCKBUF];

    let ret = unsafe {
        libc::sysctl(
            mib.as_ptr() as *mut _,
            mib.len() as _,
            &mut maxbuf as *mut libc::c_int as *mut _,
            &mut len,
            std::ptr::null_mut(),
            0,
        )
    };
    if ret == 0 {
        Some(maxbuf as usize)
    } else {
        None
    }
}

#[cfg(windows)]
fn get_platform_max_rcvbuf() -> Option<usize> {
    None
}
