//! kernel version utilities module

use rustix::system::uname;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KernelVersion {
    pub major: u32,
    pub minor: u32,
}

impl KernelVersion {
    pub fn current() -> Option<Self> {
        let uts = uname();
        let release = uts.release().to_string_lossy();
        let version = release.split('-').next()?;
        let mut parts = version.split('.');

        Some(Self {
            major: parts.next()?.parse().ok()?,
            minor: parts.next()?.parse().ok()?,
        })
    }
}
