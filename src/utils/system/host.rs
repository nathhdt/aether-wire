//! system host utilities module

use rustix::process;

/// ensures the current process is running with root privileges
pub fn ensure_root() {
    if !process::getuid().is_root() {
        eprintln!("error: aether-wire requires root privileges");
        std::process::exit(1);
    }
}
