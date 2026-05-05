//! protocol types for control channel session

use serde::{Deserialize, Serialize};

/// protocol version
pub const PROTO_VERSION: u8 = 1;

/// first message sent by the client after connecting
#[derive(Serialize, Deserialize, Debug)]
pub struct Hello {
    pub version: u8,
    pub protocol: Protocol,
    pub duration_secs: u64,
    pub n_streams: u16,
    pub verify_integrity: bool,
}

/// server answer to client hello message
#[derive(Serialize, Deserialize, Debug)]
pub struct SessionStart {
    pub session_id: u64,
    pub seed: u64,
    pub data_ports: Vec<u16>,
}

/// global statistics sent by the server after a benchmark
#[derive(Serialize, Deserialize, Debug)]
pub enum SessionStats {
    Tcp(Vec<TcpStreamStats>),
    Udp(Vec<UdpStreamStats>),
}

/// control channel session message type
#[derive(Serialize, Deserialize, Debug)]
pub enum Message {
    Hello(Hello),
    SessionStart(SessionStart),
    TestComplete,
    SessionStats(SessionStats),
    Error(String),
}

/// object containing statistics for a single TCP stream
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct TcpStreamStats {
    pub stream_id: u16,
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub duration_ns: u64,
}

/// object containing statistics for a single UDP stream
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct UdpStreamStats {
    pub stream_id: u16,
    pub packets_sent: u64,
    pub packets_received: u64,
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub packets_lost: u64,
    pub packets_out_of_order: u64,
    pub packets_duplicate: u64,
    pub jitter_mean_ms: f64,
    pub jitter_median_ms: f64,
    pub jitter_stddev_ms: f64,
    pub duration_ns: u64,
}

/// benchmark protocol type
#[derive(Serialize, Deserialize, Debug)]
pub enum Protocol {
    Tcp,
    Udp { packet_size: u16 },
}
