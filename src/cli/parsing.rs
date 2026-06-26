//! command line parsing module

use crate::{
    protocol::constants::AW_UDP_PAYLOAD_MIN_LENGTH_BYTES,
    utils::{constants::benchmark::MAX_BANDWIDTH_BPS, format::human_bps},
};

/// parses bandwidth specifications (K, M, G)
pub fn parse_bandwidth(s: &str) -> Result<u64, String> {
    let s = s.trim();
    if s.is_empty() {
        return Err("bandwidth cannot be empty".to_string());
    }

    let (num_str, unit) = match s.find(|c: char| c.is_alphabetic()) {
        Some(pos) => s.split_at(pos),
        None => (s, ""),
    };

    let num: u64 = num_str
        .parse()
        .map_err(|_| format!("invalid number: {}", num_str))?;

    if num == 0 {
        return Err("bandwidth must be positive".to_string());
    }

    let multiplier = match unit.to_uppercase().as_str() {
        "" | "BPS" => 1,
        "K" | "KBPS" => 1_000,
        "M" | "MBPS" => 1_000_000,
        "G" | "GBPS" => 1_000_000_000,
        _ => return Err(format!("unknown unit: {}. Use K, M, or G", unit)),
    };

    if num > MAX_BANDWIDTH_BPS / multiplier {
        return Err(format!(
            "bandwidth exceeds maximum allowed limit of {}",
            human_bps(MAX_BANDWIDTH_BPS)
        ));
    }

    Ok(num * multiplier)
}

/// parses duration specifications (s, m, h, d)
pub fn parse_duration(s: &str) -> Result<u64, String> {
    let s = s.trim();
    if s.is_empty() {
        return Err("duration cannot be empty".to_string());
    }

    let (num_str, unit) = match s.find(|c: char| c.is_alphabetic()) {
        Some(pos) => s.split_at(pos),
        None => (s, ""),
    };

    let num: u64 = num_str
        .parse()
        .map_err(|_| format!("invalid number: {}", num_str))?;

    if num == 0 {
        return Err("duration must be positive".to_string());
    }

    let multiplier = match unit.to_lowercase().as_str() {
        "" | "s" => 1,
        "m" => 60,
        "h" => 60 * 60,
        "d" => 60 * 60 * 24,
        _ => return Err(format!("unknown unit: {}. Use s, m, h, or d", unit)),
    };

    num.checked_mul(multiplier)
        .ok_or_else(|| "duration is too large".to_string())
}

/// parses UDP payload length in bytes
pub fn parse_udp_payload_length(s: &str) -> Result<u16, String> {
    let s = s.trim();
    if s.is_empty() {
        return Err("UDP payload length cannot be empty".to_string());
    }

    let length: u16 = s
        .parse()
        .map_err(|_| format!("invalid UDP payload length: {}", s))?;

    if length < AW_UDP_PAYLOAD_MIN_LENGTH_BYTES {
        return Err(format!(
            "UDP payload length must be at least {} bytes",
            AW_UDP_PAYLOAD_MIN_LENGTH_BYTES
        ));
    }

    Ok(length)
}
