//! utilities to print benchmark results

use crate::info_noprefix_notimestamp;
use crate::protocol::stats::{TcpStreamStats, UdpStreamStats};
use crate::utils::colors::*;
use crate::utils::format::{human_bps, human_bytes};

/// returns visible width of a string without ANSI escape sequences
fn visible_width(s: &str) -> usize {
    let mut width = 0;
    let mut chars = s.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '\x1b' {
            for ch in chars.by_ref() {
                if ch == 'm' {
                    break;
                }
            }
        } else {
            width += 1;
        }
    }

    width
}

fn print_responsive_header(role: &str, width: usize) {
    let title = format!(" {} ", role);
    let inner_width = width.saturating_sub(title.chars().count());
    let left = inner_width / 2;
    let right = inner_width - left;
    let line = format!("{}{}{}", "═".repeat(left), title, "═".repeat(right),);

    info_noprefix_notimestamp!("");
    info_noprefix_notimestamp!("      {BOLD}{BLUE}{line}{RESET}{NO_BOLD}",);
}

#[derive(Debug, Clone)]
struct TcpRow {
    stream: String,
    direction: String,
    bytes: String,
    bitrate: String,
    is_total: bool,
}

#[derive(Debug, Clone)]
struct UdpRow {
    stream: String,
    packets: String,
    bytes: String,
    bitrate: String,
    loss: String,

    jitter: Option<String>,
    ooo: Option<String>,
    dup: Option<String>,

    is_total: bool,
}

#[derive(Debug, Clone, Copy)]
struct TcpWidths {
    stream: usize,
    direction: usize,
    bytes: usize,
    bitrate: usize,
}

#[derive(Debug, Clone, Copy)]
struct UdpWidths {
    stream: usize,
    packets: usize,
    bytes: usize,
    bitrate: usize,
    loss: usize,
}

fn compute_tcp_widths(rows: &[TcpRow]) -> TcpWidths {
    TcpWidths {
        stream: rows
            .iter()
            .map(|r| visible_width(&r.stream))
            .max()
            .unwrap_or(0),

        direction: rows
            .iter()
            .map(|r| visible_width(&r.direction))
            .max()
            .unwrap_or(0),

        bytes: rows
            .iter()
            .map(|r| visible_width(&r.bytes))
            .max()
            .unwrap_or(0),

        bitrate: rows
            .iter()
            .map(|r| visible_width(&r.bitrate))
            .max()
            .unwrap_or(0),
    }
}

fn compute_udp_widths(rows: &[UdpRow]) -> UdpWidths {
    let bytes_width = rows
        .iter()
        .map(|r| {
            let normal = visible_width(&r.bytes);

            let jitter = r
                .jitter
                .as_ref()
                .map(|s| visible_width(&format!("jit: {s}")))
                .unwrap_or(0);

            normal.max(jitter)
        })
        .max()
        .unwrap_or(0);

    let bitrate_width = rows
        .iter()
        .map(|r| {
            let normal = visible_width(&r.bitrate);

            let ooo = r
                .ooo
                .as_ref()
                .map(|s| visible_width(&format!("ooo: {s}")))
                .unwrap_or(0);

            normal.max(ooo)
        })
        .max()
        .unwrap_or(0);

    let loss_width = rows
        .iter()
        .map(|r| {
            let normal = visible_width(&format!("loss {}", r.loss));

            let dup = r
                .dup
                .as_ref()
                .map(|s| visible_width(&format!("dup: {s}")))
                .unwrap_or(0);

            normal.max(dup)
        })
        .max()
        .unwrap_or(0);

    UdpWidths {
        stream: rows
            .iter()
            .map(|r| visible_width(&r.stream))
            .max()
            .unwrap_or(0),

        packets: rows
            .iter()
            .map(|r| visible_width(&r.packets))
            .max()
            .unwrap_or(0),

        bytes: bytes_width,

        bitrate: bitrate_width,

        loss: loss_width,
    }
}

fn tcp_table_width(w: TcpWidths) -> usize {
    w.stream + 3 + w.direction + 3 + w.bytes + 3 + w.bitrate
}

fn udp_table_width(w: UdpWidths) -> usize {
    w.stream + 3 + w.packets + 3 + w.bytes + 3 + w.bitrate + 3 + w.loss
}

fn render_tcp_row(row: &TcpRow, w: TcpWidths) -> String {
    let prefix = if row.is_total {
        format!("{BOLD}{PINK}")
    } else {
        MAROON.to_string()
    };

    let bitrate_color = if row.is_total { PINK } else { BLUE };

    format!(
        "{prefix}{:<stream_w$}{BLUE} │ {:>dir_w$} │ {:>bytes_w$} │ {bitrate_color}{BOLD}{:>bitrate_w$}{NO_BOLD}{BLUE}{RESET}",
        row.stream,
        row.direction,
        row.bytes,
        row.bitrate,
        stream_w = w.stream,
        dir_w = w.direction,
        bytes_w = w.bytes,
        bitrate_w = w.bitrate,
    )
}

fn render_udp_row(row: &UdpRow, w: UdpWidths) -> String {
    let prefix = if row.is_total {
        format!("{BOLD}{PINK}")
    } else {
        MAROON.to_string()
    };

    let bitrate_color = if row.is_total { PINK } else { BLUE };

    let loss_col = format!("loss {}", row.loss);

    format!(
        "{prefix}{:<stream_w$}{BLUE} │ {:>packets_w$} │ {:>bytes_w$} │ {bitrate_color}{BOLD}{:>bitrate_w$}{NO_BOLD}{BLUE} │ {:>loss_w$}{RESET}",
        row.stream,
        row.packets,
        row.bytes,
        row.bitrate,
        loss_col,
        stream_w = w.stream,
        packets_w = w.packets,
        bytes_w = w.bytes,
        bitrate_w = w.bitrate,
        loss_w = w.loss,
    )
}

fn render_udp_detail_row(row: &UdpRow, w: UdpWidths) -> Option<String> {
    let jitter = row.jitter.as_ref()?;
    let ooo = row.ooo.as_ref()?;
    let dup = row.dup.as_ref()?;

    let jitter_col = format!("jit: {jitter}");
    let ooo_col = format!("ooo: {ooo}");
    let dup_col = format!("dup: {dup}");

    Some(format!(
        "{BLUE}{:<stream_w$} │ {:>packets_w$} │ {:>bytes_w$} │ {:>bitrate_w$} │ {:>loss_w$}{RESET}",
        "",
        "",
        jitter_col,
        ooo_col,
        dup_col,
        stream_w = w.stream,
        packets_w = w.packets,
        bytes_w = w.bytes,
        bitrate_w = w.bitrate,
        loss_w = w.loss,
    ))
}

/// TCP results report
pub fn print_tcp_results(role: &str, stats: &[TcpStreamStats], is_sender: bool) {
    let direction = if is_sender { "sent" } else { "received" };

    let mut rows: Vec<TcpRow> = Vec::new();

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

        rows.push(TcpRow {
            stream: format!("stream {:>2}", s.stream_id),
            direction: direction.to_string(),
            bytes: human_bytes(bytes),
            bitrate: human_bps(bitrate),
            is_total: false,
        });

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

        rows.push(TcpRow {
            stream: "total".to_string(),
            direction: direction.to_string(),
            bytes: human_bytes(total_bytes),
            bitrate: human_bps(total_bps),
            is_total: true,
        });
    }

    let widths = compute_tcp_widths(&rows);
    let table_width = tcp_table_width(widths);

    print_responsive_header(role, table_width);
    for (idx, row) in rows.iter().enumerate() {
        if row.is_total && idx > 0 {
            info_noprefix_notimestamp!("      {BLUE}{}{RESET}", "─".repeat(table_width),);
        }

        let rendered = render_tcp_row(row, widths);
        info_noprefix_notimestamp!("      {rendered}");
    }

    info_noprefix_notimestamp!("");
}

/// UDP results report
pub fn print_udp_results(role: &str, stats: &[UdpStreamStats], is_sender: bool) {
    let mut rows: Vec<UdpRow> = Vec::new();

    let mut total_bytes: u64 = 0;
    let mut total_packets: u64 = 0;
    let mut total_lost: u64 = 0;
    let mut max_duration_ns: u64 = 0;

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

        rows.push(UdpRow {
            stream: format!("stream {:>2}", s.stream_id),
            packets: format!("{packets} pkts"),
            bytes: human_bytes(bytes),
            bitrate: human_bps(bitrate),
            loss: format!("{loss_rate:.1}%"),

            jitter: if is_sender {
                None
            } else {
                Some(format!("{}ms", s.jitter_rfc3550_ms))
            },

            ooo: if is_sender {
                None
            } else {
                Some(format!("{}", s.packets_out_of_order))
            },

            dup: if is_sender {
                None
            } else {
                Some(format!("{}", s.packets_duplicate))
            },

            is_total: false,
        });

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

        rows.push(UdpRow {
            stream: "total".to_string(),
            packets: format!("{total_packets} pkts"),
            bytes: human_bytes(total_bytes),
            bitrate: human_bps(total_bps),
            loss: format!("{total_loss_rate:.1}%"),
            jitter: None,
            ooo: None,
            dup: None,
            is_total: true,
        });
    }

    let widths = compute_udp_widths(&rows);
    let table_width = udp_table_width(widths);

    print_responsive_header(role, table_width);
    for (idx, row) in rows.iter().enumerate() {
        if row.is_total && idx > 0 {
            info_noprefix_notimestamp!("      {BLUE}{}{RESET}", "─".repeat(table_width),);
        }

        let rendered = render_udp_row(row, widths);
        info_noprefix_notimestamp!("      {rendered}");
        if let Some(detail) = render_udp_detail_row(row, widths) {
            info_noprefix_notimestamp!("      {detail}");
        }
    }

    info_noprefix_notimestamp!("");
}
