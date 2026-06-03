//! host utilities module

use rustix::process;
use std::process::ExitCode;

/// checks if the current process is running with root privileges
pub fn ensure_root() -> Result<(), ExitCode> {
    if !process::getuid().is_root() {
        eprintln!("error: aether-wire requires root privileges");
        return Err(ExitCode::FAILURE);
    }
    Ok(())
}
