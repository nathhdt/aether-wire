//! protocol messages for control channel session

use serde::{Deserialize, Serialize};

use crate::protocol::stats::{TcpStreamStats, UdpStreamStats};

/// protocol version
pub const PROTO_VERSION: u8 = 1;

/// first message sent by the client after connecting
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Hello {
    pub version: u8,
    pub session_type: SessionType,
}

/// session type to determine what the client wants to do
#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum SessionType {
    TcpBenchmark(TcpBenchmarkConfig),
    UdpBenchmark(UdpBenchmarkConfig),
    Qualify,
}

/// TCP benchmark configuration
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TcpBenchmarkConfig {
    pub duration_secs: u64,
    pub n_streams: u16,
    pub verify_integrity: Option<u64>,
    pub direction: Direction,
}

/// UDP benchmark configuration
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct UdpBenchmarkConfig {
    pub duration_secs: u64,
    pub n_streams: u16,
    pub bandwidth: u64,
    pub payload_size: u16,
}

/// traffic direction for TCP benchmark
#[derive(Clone, Copy, Debug, Eq, Deserialize, PartialEq, Serialize)]
pub enum Direction {
    Default,
    Reverse,
    Both,
    Bidirectional,
}

/// server answer to client hello message
#[derive(Debug, Deserialize, Serialize)]
pub struct SessionStart {
    pub session_id: u64,
    pub seed: u64,
    pub data_ports: Vec<u16>,
}

/// statistics sent by the server after a session
#[derive(Debug, Deserialize, Serialize)]
pub enum SessionStats {
    TcpBenchmark {
        upload: Option<Vec<TcpStreamStats>>,
        download: Option<Vec<TcpStreamStats>>,
    },
    UdpBenchmark {
        upload: Option<Vec<UdpStreamStats>>,
        download: Option<Vec<UdpStreamStats>>,
    },
    Qualify(Vec<UdpStreamStats>),
}

/// control channel session message type
#[derive(Debug, Deserialize, Serialize)]
pub enum Message {
    Hello(Hello),
    SessionStart(SessionStart),
    TestComplete,
    SessionStats(SessionStats),
    Error(String),
}
