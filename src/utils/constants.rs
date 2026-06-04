//! global application constants

/// system-related constants
pub mod system {
    /// kernel
    pub const KERNEL_MIN_MAJOR: u32 = 5;
    pub const KERNEL_MIN_MINOR: u32 = 15;
}

/// UDP-related constants
pub mod udp {
    /// maximum UDP benchmark bandwidth in bits per second
    pub const MAX_BANDWIDTH_BPS: u64 = 2_500_000_000;
}
