//! protocol types for control channel session

use serde::{Deserialize, Serialize};

/// protocol version
pub const PROTO_VERSION: u8 = 1;

/// first message sent by the client after connecting
#[derive(Serialize, Deserialize, Debug)]
pub struct Hello {
    pub version: u8,
    pub duration_secs: u64,
    pub n_streams: u16,
}

/// server answer to client hello message
#[derive(Serialize, Deserialize, Debug)]
pub struct SessionStart {
    pub session_id: u64,
    pub seed: u64,
    pub data_port: u16,
}

/// global statistics sent by the server after a benchmark
#[derive(Serialize, Deserialize, Debug)]
pub struct SessionStats {
    pub streams: Vec<StreamStats>,
}

///control channel session message type
#[derive(Serialize, Deserialize, Debug)]
pub enum Message {
    Hello(Hello),
    SessionStart(SessionStart),
    SessionStats(SessionStats),
    Error(String),
}

/// object containing statistics for a single stream
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct StreamStats {
    pub stream_id: u16,
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub duration_ns: u64,
}
