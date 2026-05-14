//! UDP statistics compute module

use anyhow::Result;

use crate::protocol::stats::UdpStreamStats;

/// UDP stream runtime statistics
#[derive(Default)]
pub struct StreamState {
    pub packets_recv: u64,
    pub bytes_received: u64,

    pub first_recv_ts: Option<u64>,

    pub last_send_ts: Option<u64>,
    pub last_recv_ts: Option<u64>,

    // RFC3550 interarrival jitter estimator
    pub jitter_ns: f64,
}

/// statistics compute for received UDP packets
pub fn compute_stats(streams: Vec<StreamState>) -> Result<Vec<UdpStreamStats>> {
    let mut stats = Vec::new();

    for (stream_id, stream) in streams.iter().enumerate() {
        let duration_ns = match (stream.first_recv_ts, stream.last_recv_ts) {
            (Some(first), Some(last)) => last.saturating_sub(first),
            _ => 0,
        };

        stats.push(UdpStreamStats {
            stream_id: stream_id as u16,

            bytes_sent: 0,
            bytes_received: stream.bytes_received,

            packets_sent: 0,
            packets_recv: stream.packets_recv,

            packets_lost: 0,
            packets_out_of_order: 0,
            packets_duplicate: 0,

            // jitter to milliseconds
            jitter_rfc3550_ms: (stream.jitter_ns / 1_000_000.0) as u64,

            duration_ns,
        });
    }

    Ok(stats)
}
