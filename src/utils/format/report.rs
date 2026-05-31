//! utilities to print benchmark results

use crate::info_noprefix;
use crate::protocol::stats::{TcpStreamStats, UdpStreamStats};
use crate::utils::format::bytes_formatting::{human_bps, human_bytes};

fn print_header(role: &str) {
    info_noprefix!("");
    info_noprefix!("");
    info_noprefix!("{role}");
}

/// TCP results report
pub fn print_tcp_results(role: &str, stats: &[TcpStreamStats], is_sender: bool) {
    let direction = if is_sender { "sent" } else { "received" };

    let mut total_bytes: u64 = 0;
    let mut max_duration_ns: u64 = 0;

    print_header(role);

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

        info_noprefix!(
            "{:<10} {:<8} {:>12} {:>12}",
            format!("stream {:>2}", s.stream_id),
            direction,
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

        info_noprefix!("");

        info_noprefix!(
            "{:<10} {:<8} {:>12} {:>12}",
            "total",
            direction,
            human_bytes(total_bytes),
            human_bps(total_bps),
        );
    }
}

/// UDP results report
pub fn print_udp_results(role: &str, stats: &[UdpStreamStats], is_sender: bool) {
    let mut total_bytes: u64 = 0;
    let mut total_packets: u64 = 0;
    let mut total_lost: u64 = 0;
    let mut max_duration_ns: u64 = 0;

    print_header(role);

    for s in stats {
        let secs = s.duration_ns as f64 / 1_000_000_000.0;

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

        let bitrate = if secs > 0.0 {
            (bytes as f64) * 8.0 / secs
        } else {
            0.0
        };

        let loss_rate = if is_sender {
            0.0
        } else {
            let expected = packets + s.packets_lost;

            if expected > 0 {
                (s.packets_lost as f64 / expected as f64) * 100.0
            } else {
                0.0
            }
        };

        info_noprefix!(
            "{:<10} {:>12} {:>12} {:>12} loss {}",
            format!("stream {:>2}", s.stream_id),
            format!("{packets} pkts"),
            human_bytes(bytes),
            human_bps(bitrate),
            format!("{loss_rate:.1}%"),
        );

        if !is_sender {
            info_noprefix!(
                "           jitter {}ms   ooo {}   dup {}",
                s.jitter_rfc3550_ms,
                s.packets_out_of_order,
                s.packets_duplicate,
            );
        }

        total_bytes += bytes;
        total_packets += packets;
        total_lost += s.packets_lost;

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

        let total_loss_rate = if !is_sender {
            let expected = total_packets + total_lost;

            if expected > 0 {
                (total_lost as f64 / expected as f64) * 100.0
            } else {
                0.0
            }
        } else {
            0.0
        };

        info_noprefix!("");

        info_noprefix!(
            "{:<10} {:>12} {:>12} {:>12} loss {}",
            "total",
            format!("{total_packets} pkts"),
            human_bytes(total_bytes),
            human_bps(total_bps),
            format!("{total_loss_rate:.1}%"),
        );
    }
}
