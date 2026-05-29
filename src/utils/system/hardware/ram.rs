//! RAM utilities

use std::process::Command;

/// returns total physical memory of the system
pub fn get_total_memory_bytes() -> Option<u64> {
    get_total_memory_impl()
}

#[cfg(target_os = "linux")]
fn get_total_memory_impl() -> Option<u64> {
    use std::fs;
    let content = fs::read_to_string("/proc/meminfo").ok()?;
    for line in content.lines() {
        if let Some(rest) = line.strip_prefix("MemTotal:") {
            let kb: u64 = rest.split_whitespace().next()?.parse().ok()?;
            return Some(kb * 1024);
        }
    }
    None
}

#[cfg(target_os = "macos")]
fn get_total_memory_impl() -> Option<u64> {
    let output = Command::new("sysctl")
        .args(["-n", "hw.memsize"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    str::from_utf8(&output.stdout).ok()?.trim().parse().ok()
}

#[cfg(target_os = "windows")]
fn get_total_memory_impl() -> Option<u64> {
    let output = Command::new("wmic")
        .args(["ComputerSystem", "get", "TotalPhysicalMemory", "/value"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let text = str::from_utf8(&output.stdout).ok()?;
    for line in text.lines() {
        if let Some(rest) = line.strip_prefix("TotalPhysicalMemory=") {
            return rest.trim().parse().ok();
        }
    }
    None
}
