//! UDP stream management for benchmark mode

#![allow(dead_code)]

use crate::protocol::stats::UdpStreamStats;
use anyhow::Result;

/// runs a multi-stream UDP benchmark
pub fn run_udp_benchmark(
    _server: std::net::Ipv4Addr,
    _port: u16,
    _n_streams: u16,
    _session_seed: u64,
    _duration: std::time::Duration,
    _bandwidth: u64,
    _payload_size: u16,
) -> Result<Vec<UdpStreamStats>> {
    // TODO: implement UDP benchmark logic
    Ok(Vec::new())
}
