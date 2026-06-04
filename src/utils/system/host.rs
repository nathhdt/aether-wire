//! system host utilities module

use rustix::process;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemlockLimitValue {
    Unlimited,
    Bytes(u64),
}

/// ensures the current process is running with root privileges
pub fn ensure_root() {
    if !process::getuid().is_root() {
        eprintln!("error: aether-wire requires root privileges");
        std::process::exit(1);
    }
}

/// retrieves memlock hard limit
pub fn get_memlock_limit() -> MemlockLimitValue {
    match process::getrlimit(process::Resource::Memlock).maximum {
        None => MemlockLimitValue::Unlimited,
        Some(value) => MemlockLimitValue::Bytes(value),
    }
}
