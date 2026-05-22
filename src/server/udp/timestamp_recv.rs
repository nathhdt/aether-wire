//! kernel-level UDP receive timestamps

use std::io;
use std::net::UdpSocket;
use std::time::Instant;

#[cfg(target_os = "macos")]
use crate::socket::so_timestamp::enable_so_timestamp;
#[cfg(target_os = "linux")]
use crate::socket::so_timestampns::enable_so_timestampns;
#[cfg(unix)]
use crate::utils::system::clock::clock_realtime_ns;

/// session-scoped timestamp receiver
pub struct TimestampReceiver {
    start: Instant,
    #[cfg(unix)]
    epoch_base_ns: u64,
}

/// timestamp initialization on the socket
impl TimestampReceiver {
    pub fn new(sock: &UdpSocket) -> (Self, bool) {
        let start = Instant::now();

        #[cfg(target_os = "linux")]
        {
            let epoch_base_ns = clock_realtime_ns();
            let enabled = enable_so_timestampns(sock);
            (
                Self {
                    start,
                    epoch_base_ns,
                },
                enabled,
            )
        }

        #[cfg(target_os = "macos")]
        {
            let epoch_base_ns = clock_realtime_ns();
            let enabled = enable_so_timestamp(sock);
            (
                Self {
                    start,
                    epoch_base_ns,
                },
                enabled,
            )
        }

        #[cfg(windows)]
        {
            let _ = sock;
            (Self { start }, false)
        }
    }

    /// UDP packet receiver
    #[inline]
    pub fn recv(&self, sock: &UdpSocket, buf: &mut [u8]) -> io::Result<(usize, u64)> {
        platform::recv(self, sock, buf)
    }
}

// Linux timestamp receiver
#[cfg(target_os = "linux")]
mod platform {
    use libc::SCM_TIMESTAMP;

    use super::*;
    use std::mem;
    use std::os::fd::AsRawFd;

    const SCM_TIMESTAMPNS: libc::c_int = linc::SO_TIMESTAMPNS;

    #[inline]
    pub fn recv(
        rx: &TimestampReceiver,
        sock: &UdpSocket,
        buf: &mut [u8],
    ) -> io::Result<(usize, u64)> {
        let fd = sock.as_raw_fd();

        let mut iov = libc::iovec {
            iov_base: buf.as_mut_ptr().cast(),
            iov_len: buf.len(),
        };

        let mut control = [0u8; 128];

        let mut msg: libc::msghdr = unsafe { mem::zeroed() };
        msg.msg_iov = &mut iov;
        msg.msg_iovlen = 1;
        msg.msg_control = control.as_mut_ptr().cast();
        msg.msg_controllen = control.len();

        let n = unsafe { libc::recvmsg(fd, &mut msg, O) };
        if n < 0 {
            return Err(io::Error::last_os_error());
        }

        let ts = extract_kernel_ts(&msg, rx.epoch_base_ns)
            .unwrap_or_else(|| rx.start.elapsed().as_nanos() as u64);

        Ok((n as usize, ts))
    }

    fn extract_kernel_ts(msg: &libc::msghdr, epoch_base_ns: u64) -> Option<u64> {
        unsafe {
            let mut cmsg = libc::CMSG_FIRSTHDR(msg);
            while !cmsg.is_null() {
                let hdr = &*cmsg;
                if hdr.cmsg_level == libc::SOL_SOCKET && hdr.cmsg_type == SCM_TIMESTAMPNS {
                    let ts = &*libc::CMSG_DATA(cmsg).cast::<libc::timespec>();
                    let abs_ns = ts.tv_sec as u64 * 1_000_000_000 + ts.tv_nsec as u64;
                    return Some(abs_ns.saturating_sub(epoch_base_ns));
                }
                cmsg = libc::CMSG_NXTHDR(msg, cmsg);
            }
        }
        None
    }
}

// macOS timestamp receiver
#[cfg(target_os = "macos")]
mod platform {
    use super::*;
    use std::mem;
    use std::os::fd::AsRawFd;

    #[inline]
    pub fn recv(
        rx: &TimestampReceiver,
        sock: &UdpSocket,
        buf: &mut [u8],
    ) -> io::Result<(usize, u64)> {
        let fd = sock.as_raw_fd();

        let mut iov = libc::iovec {
            iov_base: buf.as_mut_ptr().cast(),
            iov_len: buf.len() as _,
        };

        let mut control = [0u8; 128];

        let mut msg: libc::msghdr = unsafe { mem::zeroed() };
        msg.msg_iov = &mut iov;
        msg.msg_iovlen = 1;
        msg.msg_control = control.as_mut_ptr().cast();
        msg.msg_controllen = control.len() as _;

        let n = unsafe { libc::recvmsg(fd, &mut msg, 0) };
        if n < 0 {
            return Err(io::Error::last_os_error());
        }

        let ts = extract_kernel_ts(&msg, rx.epoch_base_ns)
            .unwrap_or_else(|| rx.start.elapsed().as_nanos() as u64);

        Ok((n as usize, ts))
    }

    fn extract_kernel_ts(msg: &libc::msghdr, epoch_base_ns: u64) -> Option<u64> {
        unsafe {
            let mut cmsg = libc::CMSG_FIRSTHDR(msg);
            while !cmsg.is_null() {
                let hdr = &*cmsg;
                if hdr.cmsg_level == libc::SOL_SOCKET && hdr.cmsg_type == libc::SCM_TIMESTAMP {
                    let tv = &*libc::CMSG_DATA(cmsg).cast::<libc::timeval>();
                    let abs_ns = tv.tv_sec as u64 * 1_000_000_000 + tv.tv_usec as u64 * 1_000;
                    return Some(abs_ns.saturating_sub(epoch_base_ns));
                }
                cmsg = libc::CMSG_NXTHDR(msg, cmsg);
            }
        }
        None
    }
}

// Windows timestamp receiver
#[cfg(windows)]
mod platform {
    use super::*;

    #[inline]
    pub fn recv(
        rx: &TimestampReceiver,
        sock: &UdpSocket,
        buf: &mut [u8],
    ) -> io::Result<(usize, u64)> {
        let (n, _) = sock.recv_from(buf)?;
        let ts = rx.start.elapsed().as_nanos() as u64;
        Ok((n, ts))
    }
}
