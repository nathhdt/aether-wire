//! command line parsing module

use crate::utils::{constants::udp::MAX_BANDWIDTH_BPS, format::human_bps};

/// parses bandwidth specifications (K, M, G)
pub fn parse_bandwidth(s: &str) -> Result<u64, String> {
    let s = s.trim();
    if s.is_empty() {
        return Err("bandwidth cannot be empty".to_string());
    }

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

    if num > MAX_BANDWIDTH_BPS / multiplier {
        return Err(format!(
            "bandwidth exceeds maximum allowed limit of {}",
            human_bps(MAX_BANDWIDTH_BPS)
        ));
    }

    Ok(num * multiplier)
}
