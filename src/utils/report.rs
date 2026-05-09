//! utilities to print benchmark results

use crate::protocol::stats::TcpStreamStats;
use crate::utils::format::{human_bps, human_bytes};

/// prints per-stream stats and total for multi-stream benchmarks
pub fn print_results(role: &str, stats: &[TcpStreamStats], is_sender: bool) {
    let label = if is_sender { "sent    " } else { "received" };

    println!();
    println!("======== {role} ========");

    let mut total_bytes: u64 = 0;
    let mut max_duration_ns: u64 = 0;

    for s in stats {
        let secs = s.duration_ns as f64 / 1_000_000_000.0;
        let bytes = if is_sender {
            s.bytes_sent
        } else {
            s.bytes_received
        };
        let bitrate = if secs > 0.0 {
            (bytes as f64) * 8.0 / secs
        } else {
            0.0
        };

        println!(
            "  stream {:>2} — {} {} — {}",
            s.stream_id,
            label,
            human_bytes(bytes),
            human_bps(bitrate),
        );

        total_bytes += bytes;
        if s.duration_ns > max_duration_ns {
            max_duration_ns = s.duration_ns;
        }
    }

    if stats.len() > 1 {
        let secs = max_duration_ns as f64 / 1_000_000_000.0;
        let total_bps = if secs > 0.0 {
            (total_bytes as f64) * 8.0 / secs
        } else {
            0.0
        };
        println!("  ────────────────────────────────────────");
        println!(
            "  total     — {} {} — {}",
            label,
            human_bytes(total_bytes),
            human_bps(total_bps),
        );
    }
}
