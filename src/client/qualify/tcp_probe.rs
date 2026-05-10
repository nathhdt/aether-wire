//! qualify mode step 1 - TCP probe

use anyhow::Result;
use std::net::Ipv4Addr;
use std::time::Duration;

use crate::client::benchmark::client::{TcpBenchmarkParameters, run_tcp_silent};
use crate::info;
use crate::protocol::messages::Direction;
use crate::protocol::stats::TcpStreamStats;
use crate::utils::format::human_bps;

/// runs TCP probe to establish reference throughput (Vref)
pub fn tcp_probe(server: Ipv4Addr, port: u16) -> Result<f64> {
    info!("qualify", "step 1: TCP probe");

    // test 1: single stream
    info!("qualify", "  running single stream test (15s)...");
    let single_params = TcpBenchmarkParameters {
        server,
        port,
        duration: Duration::from_secs(15),
        n_streams: 1,
        verify_integrity: false,
        direction: Direction::Default,
    };

    let (_, single_server_stats) = run_tcp_silent(single_params)?;
    let throughput_single =
        calculate_throughput(&single_server_stats.expect("server should return stats"))?;

    info!(
        "qualify",
        "  single stream: {}",
        human_bps(throughput_single)
    );

    // test 2: multi stream
    info!("qualify", "  running multi stream test (4 streams, 15s)...");
    let multi_params = TcpBenchmarkParameters {
        server,
        port,
        duration: Duration::from_secs(15),
        n_streams: 4,
        verify_integrity: false,
        direction: Direction::Default,
    };

    let (_, multi_server_stats) = run_tcp_silent(multi_params)?;
    let throughput_multi =
        calculate_throughput(&multi_server_stats.expect("server should return stats"))?;

    info!("qualify", "  multi stream: {}", human_bps(throughput_multi));

    // Vref calculation
    let vref = throughput_single.max(throughput_multi);

    info!(
        "qualify",
        "  Vref = {} (reference throughput established)",
        human_bps(vref)
    );
    info!("qualify", "step 1 complete");

    Ok(vref)
}

/// calculates total throughput from stream statistics
fn calculate_throughput(stats: &[TcpStreamStats]) -> Result<f64> {
    let total_bytes: u64 = stats.iter().map(|s| s.bytes_received).sum();
    let max_duration_ns = stats.iter().map(|s| s.duration_ns).max().unwrap_or(0);

    let secs = max_duration_ns as f64 / 1_000_000_000.0;

    Ok(if secs > 0.0 {
        (total_bytes as f64) * 8.0 / secs
    } else {
        0.0
    })
}
