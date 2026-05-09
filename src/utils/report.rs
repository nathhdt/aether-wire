//! utilities to print benchmark results

use crate::protocol::stats::TcpStreamStats;
use crate::utils::colors::*;
use crate::utils::format::{human_bps, human_bytes};

// import des macros
use crate::info_noprefix;

pub fn print_results(role: &str, stats: &[TcpStreamStats], is_sender: bool) {
    const HEADER_WIDTH: usize = 53;
    const SEPARATOR_WIDTH: usize = 50;

    let direction = if is_sender { "sent" } else { "received" };

    let title = format!(" {} ", role);
    let pad = HEADER_WIDTH.saturating_sub(title.len());
    let left = pad / 2;
    let right = pad - left;

    info_noprefix!("");
    info_noprefix!(
        "   {BOLD}{CYAN}{}{title}{}{RESET}{NO_BOLD}",
        "═".repeat(left),
        "═".repeat(right),
    );

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

        info_noprefix!(
            "      {YELLOW}stream {:>2} {CYAN}│ {:>8} │ {:>12} │ {BOLD}{:>12}{NO_BOLD}",
            s.stream_id,
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

        info_noprefix!("      {CYAN}{}{RESET}", "─".repeat(SEPARATOR_WIDTH));

        info_noprefix!(
            "      {BOLD}{GREEN}total     {CYAN}│ {:>8} │ {:>12} │ {:>12}{NO_BOLD}",
            direction,
            human_bytes(total_bytes),
            human_bps(total_bps),
        );
    }

    info_noprefix!("");
}
