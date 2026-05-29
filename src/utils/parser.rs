//! TUI and command line parsing module

use std::net::Ipv4Addr;
use std::time::Duration;

use crate::utils::format::bytes_formatting::human_bytes;
use crate::utils::system::hardware::cpu::cpu_cores_count;
use crate::utils::system::hardware::ram::get_total_memory_bytes;

/// checks for a minimal duration of 1s
pub fn parse_duration_min_1s(s: &str) -> Result<Duration, String> {
    let d = humantime::parse_duration(s).map_err(|e| e.to_string())?;

    if d < Duration::from_secs(1) {
        return Err("duration must be at least 1s".to_string());
    }

    Ok(d)
}

/// validates the provided server IPv4 is an actual reachable host
pub fn parse_server_ipv4(s: &str) -> Result<Ipv4Addr, String> {
    let ip: Ipv4Addr = s
        .parse()
        .map_err(|_| format!("{s} is not a valid IPv4 address"))?;

    if ip.is_unspecified() {
        return Err("0.0.0.0 is not a valid host address".into());
    }

    if ip.is_multicast() {
        return Err("multicast addresses are not valid hosts".into());
    }

    if ip.octets() == [255, 255, 255, 255] {
        return Err("broadcast addresses is not a valid host".into());
    }

    Ok(ip)
}

/// parses bandwidth specifications (K, M, G)
pub fn parse_bandwidth(s: &str) -> Result<u64, String> {
    const MAX_BANDWIDTH: u64 = 5_000_000_000; // 5 Gbit/s

    let s = s.trim();
    if s.is_empty() {
        return Err("bandwidth cannot be empty".to_string());
    }

    // extracts number and unit
    let (num_str, unit) = if let Some(pos) = s.find(|c: char| c.is_alphabetic()) {
        (&s[..pos], &s[pos..])
    } else {
        (s, "")
    };

    let num: u64 = num_str
        .parse()
        .map_err(|_| format!("invalid number: {}", num_str))?;

    let multiplier = match unit.to_uppercase().as_str() {
        "" | "BPS" => 1,
        "K" | "KBPS" => 1_000,
        "M" | "MBPS" => 1_000_000,
        "G" | "GBPS" => 1_000_000_000,
        _ => return Err(format!("unknown unit: {}. Use K, M, or G", unit)),
    };

    // total bandwidth
    let total_bandwidth = num * multiplier;

    // checks max bandwidth
    if total_bandwidth > MAX_BANDWIDTH {
        return Err("bandwidth exceeds maximum allowed limit of 5 Gbit/s".to_string());
    }

    Ok(total_bandwidth)
}

// parses asked stream number
pub fn parse_stream_number(s: &str) -> Result<u16, String> {
    let n: u16 = s.parse().map_err(|_| "invalid stream count".to_string())?;

    // max streams is from n CPU threads with a n_max of 32
    let max_streams = cpu_cores_count();

    if n == 0 {
        return Err("stream count must be at least 1".to_string());
    }

    if n as usize > max_streams {
        return Err(format!(
            "stream count exceeds available CPU cores ({max_streams})"
        ));
    }

    Ok(n)
}

/// parses size bytes (B, K, M, G) for multiple cases
fn parse_size_bytes(s: &str) -> Result<u64, String> {
    let s = s.trim();

    if s.is_empty() {
        return Err("size cannot be empty".into());
    }

    let (num_str, unit) = if let Some(pos) = s.find(|c: char| c.is_alphabetic()) {
        (&s[..pos], &s[pos..])
    } else {
        (s, "")
    };

    let num: u64 = num_str
        .parse()
        .map_err(|_| format!("invalid number: {num_str}"))?;

    let multiplier = match unit.to_uppercase().as_str() {
        "" | "B" => 1,
        "K" | "KB" | "KIB" => 1024,
        "M" | "MB" | "MIB" => 1024 * 1024,
        "G" | "GB" | "GIB" => 1024 * 1024 * 1024,
        _ => return Err(format!("unknown unit: {unit}. Use B, K, M, or G")),
    };

    num.checked_mul(multiplier)
        .ok_or_else(|| "size overflow".to_string())
}

/// parses memory size for UDP receiving buffer
pub fn parse_udp_buf_mem_size(s: &str) -> Result<u64, String> {
    const MAX_SIZE: u64 = 256 * 1024 * 1024;

    let size = parse_size_bytes(s)?;

    if size == 0 {
        return Err("buffer size must be at least 1 byte".into());
    }

    if size > MAX_SIZE {
        return Err("buffer size exceeds maximum allowed limit of 256 MiB".into());
    }

    Ok(size)
}

/// parses --verify option buffer size
pub fn parse_verify_size(s: &str) -> Result<u64, String> {
    let size = parse_size_bytes(s)?;

    if size == 0 {
        return Err("verify size must be at least 1 byte".into());
    }

    if let Some(max_size) = get_total_memory_bytes()
        && size > max_size
    {
        return Err(format!(
            "verify size must be lower than available memory ({})",
            human_bytes(max_size)
        ));
    }

    Ok(size)
}
