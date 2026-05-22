//! hardware utilities

use std::process::Command;
use std::str;
use std::thread;

/// returns the number of available CPU cores for benchmark purposes
pub fn cpu_cores_count() -> usize {
    get_apple_silicon_perf_cores().unwrap_or_else(|| {
        thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(1)
            .min(32)
    })
}

#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn get_apple_silicon_perf_cores() -> Option<usize> {
    let output = Command::new("sysctl")
        .args(["-n", "hw.perflevel0.logicalcpu"])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    str::from_utf8(&output.stdout).ok()?.trim().parse().ok()
}

#[cfg(not(all(target_os = "macos", target_arch = "aarch64")))]
fn get_apple_silicon_perf_cores() -> Option<usize> {
    None
}
