//! statistics structures for benchmark sessions

use serde::{Deserialize, Serialize};

/// object containing statistics for a single TCP stream
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct TcpStreamStats {
    pub stream_id: u16,
    pub bytes_sent: u64,
    pub bytes_recv: u64,
    pub duration_ns: u64,
}

/// object containing statistics for a single UDP stream
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct UdpStreamStats {
    pub stream_id: u16,
    pub bytes_sent: u64,
    pub bytes_recv: u64,
    pub packets_sent: u64,
    pub packets_recv: u64,
    pub packets_lost: u64,
    pub packets_out_of_order: u64,
    pub packets_duplicate: u64,
    pub jitter_mean_ms: u64,
    pub jitter_median_ms: u64,
    pub jitter_stddev_ms: u64,
    pub duration_ns: u64,
}
