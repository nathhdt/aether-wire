//! TUI result formatting for UDP benchmark

use crate::protocol::stats::UdpStreamStats;
use crate::utils::format::bytes_formatting::{human_bps, human_bytes};

/// formats UDP stats for TUI display
pub fn format_udp_benchmark_result(stats: &[UdpStreamStats], is_sender: bool) -> Vec<String> {
    let mut lines = Vec::new();
    let mut total_bytes: u64 = 0;
    let mut total_packets: u64 = 0;
    let mut total_lost: u64 = 0;
    let mut max_ns: u64 = 0;

    for s in stats {
        let bytes = if is_sender {
            s.bytes_sent
        } else {
            s.bytes_received
        };
        let packets = if is_sender {
            s.packets_sent
        } else {
            s.packets_recv
        };

        let secs = s.duration_ns as f64 / 1e9;
        let bps = if secs > 0.0 {
            bytes as f64 * 8.0 / secs
        } else {
            0.0
        };

        if is_sender {
            lines.push(format!(
                "  stream {:>2}  {}  {}  ({} pkts sent)",
                s.stream_id,
                human_bps(bps),
                human_bytes(bytes),
                packets,
            ));
        } else {
            let expected = packets + s.packets_lost;
            let loss = if expected > 0 {
                s.packets_lost as f64 / expected as f64 * 100.0
            } else {
                0.0
            };

            lines.push(format!(
                "  stream {:>2}  {}  {}  loss {:.1}%  jitter {}ms",
                s.stream_id,
                human_bps(bps),
                human_bytes(bytes),
                loss,
                s.jitter_rfc3550_ms,
            ));
        }

        total_bytes += bytes;
        total_packets += packets;
        total_lost += s.packets_lost;
        max_ns = max_ns.max(s.duration_ns);
    }

    if stats.len() > 1 {
        let secs = max_ns as f64 / 1e9;
        let total_bps = if secs > 0.0 {
            total_bytes as f64 * 8.0 / secs
        } else {
            0.0
        };

        if is_sender {
            lines.push(format!(
                "  total     {}  {}  ({} pkts sent)",
                human_bps(total_bps),
                human_bytes(total_bytes),
                total_packets,
            ));
        } else {
            let total_expected = total_packets + total_lost;
            let total_loss = if total_expected > 0 {
                total_lost as f64 / total_expected as f64 * 100.0
            } else {
                0.0
            };

            lines.push(format!(
                "  total     {}  {}  loss {:.1}%",
                human_bps(total_bps),
                human_bytes(total_bytes),
                total_loss,
            ));
        }
    }

    lines
}
