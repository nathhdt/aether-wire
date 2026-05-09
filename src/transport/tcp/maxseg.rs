//! TCP_MAXSEG

use anyhow::Result;
use std::net::TcpStream;

use crate::bail_error;

/// returns the current TCP_MAXSEG (or MSS) for a specific TCP stream
pub fn get_tcp_maxseg(socket: &TcpStream) -> Result<u16> {
    platform::tcp_maxseg(socket)
}

/// gets TCP_MAXSEG for Unix-based OSes
#[cfg(unix)]
mod platform {
    use super::*;

    use std::os::fd::AsRawFd;

    pub fn tcp_maxseg(socket: &TcpStream) -> Result<u16> {
        let fd = socket.as_raw_fd();

        let mut mss: libc::c_int = 0;
        let mut len = std::mem::size_of_val(&mss) as libc::socklen_t;

        let ret = unsafe {
            libc::getsockopt(
                fd,
                libc::IPPROTO_TCP,
                libc::TCP_MAXSEG,
                (&mut mss as *mut libc::c_int).cast(),
                &mut len,
            )
        };

        if ret != 0 {
            bail_error!("aw", "getsockopt(TCP_MAXSEG) failed");
        }

        Ok(u16::try_from(mss)?)
    }
}

/// gets TCP_MAXSEG for Windows-based OSes
#[cfg(windows)]
mod platform {
    use super::*;

    use std::os::windows::io::AsRawSocket;
    use windows_sys::Win32::Networking::WinSock::{IPPROTO_TCP, SOCKET, TCP_MAXSEG, getsockopt};

    pub fn tcp_maxseg(socket: &TcpStream) -> Result<u16> {
        let sock = socket.as_raw_socket() as SOCKET;

        let mut mss: i32 = 0;
        let mut len = std::mem::size_of_val(&mss) as i32;

        let ret = unsafe {
            getsockopt(
                sock,
                IPPROTO_TCP as i32,
                TCP_MAXSEG as i32,
                (&mut mss as *mut i32).cast(),
                &mut len,
            )
        };

        if ret != 0 {
            bail_error!("aw", "getsockopt(TCP_MAXSEG) failed");
        }

        Ok(u16::try_from(mss)?)
    }
}
