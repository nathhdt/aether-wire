//! socket receive buffer configuration (cross-platform) via SO_RCVBUF

use std::io;
use std::mem;
use std::net::UdpSocket;

#[cfg(unix)]
use std::os::fd::AsRawFd;
#[cfg(windows)]
use std::os::windows::io::AsRawSocket;

/// clamps the target size to platform limits and applies it
pub fn set_so_rcvbuf(sock: &UdpSocket, target_bytes: usize) -> std::io::Result<usize> {
    #[cfg(unix)]
    let fd = sock.as_raw_fd() as _;
    #[cfg(windows)]
    let fd = sock.as_raw_socket() as _;

    let max_bytes = get_platform_max_rcvbuf().unwrap_or(target_bytes);
    let desired = target_bytes.min(max_bytes);

    // Linux internally doubles SO_RCVBUF values
    #[cfg(target_os = "linux")]
    let desired = desired / 2;

    let desired_ffi: libc::c_int = desired
        .try_into()
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "SO_RCVBUF too large"))?;

    let ret = unsafe {
        libc::setsockopt(
            fd,
            libc::SOL_SOCKET,
            libc::SO_RCVBUF,
            (&desired_ffi as *const libc::c_int).cast(),
            mem::size_of_val(&desired_ffi) as libc::socklen_t,
        )
    };

    if ret != 0 {
        return Err(io::Error::last_os_error());
    }

    let mut actual: libc::c_int = 0;
    let mut len = mem::size_of_val(&actual) as libc::socklen_t;

    let ret = unsafe {
        libc::getsockopt(
            fd,
            libc::SOL_SOCKET,
            libc::SO_RCVBUF,
            (&mut actual as *mut libc::c_int).cast(),
            &mut len,
        )
    };

    if ret != 0 {
        return Err(io::Error::last_os_error());
    }

    Ok(actual as usize)
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
