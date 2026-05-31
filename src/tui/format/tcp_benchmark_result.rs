//! TUI result formatting for TCP benchmark

use crate::protocol::stats::TcpStreamStats;
use crate::utils::format::bytes_formatting::{human_bps, human_bytes};

/// formats TCP stats for TUI display
pub fn format_tcp_result(stats: &[TcpStreamStats], is_sender: bool) -> Vec<String> {
    let direction = if is_sender { "sent" } else { "received" };
    let mut lines = Vec::new();
    let mut total_bytes: u64 = 0;
    let mut max_ns: u64 = 0;

    for s in stats {
        let bytes = if is_sender {
            s.bytes_sent
        } else {
            s.bytes_received
        };
        let secs = s.duration_ns as f64 / 1e9;
        let bps = if secs > 0.0 {
            bytes as f64 * 8.0 / secs
        } else {
            0.0
        };

        lines.push(format!(
            "  stream {:>2}  {}  {}  ({direction})",
            s.stream_id,
            human_bps(bps),
            human_bytes(bytes),
        ));

        total_bytes += bytes;
        max_ns = max_ns.max(s.duration_ns);
    }

    if stats.len() > 1 {
        let secs = max_ns as f64 / 1e9;
        let total_bps = if secs > 0.0 {
            total_bytes as f64 * 8.0 / secs
        } else {
            0.0
        };

        lines.push(format!(
            "  total     {}  {}  ({direction})",
            human_bps(total_bps),
            human_bytes(total_bytes),
        ));
    }

    lines
}
