//! collection of utility functions

use rand_core::{OsRng, RngCore};

use crate::proto::StreamStats;

/// converts bytes per second data measure to human-readable format
pub fn human_bps(bps: f64) -> String {
    const K: f64 = 1_000.0;
    const M: f64 = 1_000_000.0;
    const G: f64 = 1_000_000_000.0;

    if bps >= G {
        format!("{:.2} Gbit/s", bps / G)
    } else if bps >= M {
        format!("{:.2} Mbit/s", bps / M)
    } else if bps >= K {
        format!("{:.2} Kbit/s", bps / K)
    } else {
        format!("{bps:.0} bit/s")
    }
}

/// converts bytes data measure to human-readable format
pub fn human_bytes(b: u64) -> String {
    const K: f64 = 1024.0;
    const M: f64 = 1024.0 * 1024.0;
    const G: f64 = 1024.0 * 1024.0 * 1024.0;

    let bf = b as f64;

    if bf >= G {
        format!("{:.2} GiB", bf / G)
    } else if bf >= M {
        format!("{:.2} MiB", bf / M)
    } else if bf >= K {
        format!("{:.2} KiB", bf / K)
    } else {
        format!("{b} B")
    }
}

/// gives a random u64
pub fn rand_u64() -> u64 {
    OsRng.next_u64()
}

/// prints per-stream stats and total for multi-stream benchmarks
pub fn print_results(role: &str, stats: &[StreamStats], is_sender: bool) {
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
            "  total    — {} {} — {}",
            label,
            human_bytes(total_bytes),
            human_bps(total_bps),
        );
    }
}
