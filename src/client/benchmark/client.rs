//! aether-wire benchmark mode client

use anyhow::Result;
use std::net::{IpAddr, SocketAddr, TcpStream};

use crate::client::benchmark::stream;
use crate::protocol::messages::{
    BenchmarkConfig, Direction, Hello, Message, PROTO_VERSION, SessionStats, SessionType,
};
use crate::protocol::stats::TcpStreamStats;
use crate::protocol::wire;
use crate::utils::report::print_results;
use crate::{bail_error, info};

/// client benchmark arguments structure
pub struct BenchmarkParameters {
    pub server: std::net::Ipv4Addr,
    pub port: u16,
    pub duration: std::time::Duration,
    pub n_streams: u16,
    pub verify_integrity: bool,
    pub direction: Direction,
}

/// benchmark result containing upload and download statistics
pub type BenchmarkResult = (Option<Vec<TcpStreamStats>>, Option<Vec<TcpStreamStats>>);

/// internal benchmark execution, returns stats without printing
fn run_internal(args: BenchmarkParameters) -> Result<BenchmarkResult> {
    // control channel session establishment
    let ctrl_addr = SocketAddr::new(IpAddr::V4(args.server), args.port);
    let mut ctrl_sock = TcpStream::connect(ctrl_addr)?;

    // hello message with protocol
    let hello = Message::Hello(Hello {
        version: PROTO_VERSION,
        session_type: SessionType::Benchmark(BenchmarkConfig {
            duration_secs: args.duration.as_secs(),
            n_streams: args.n_streams,
            verify_integrity: args.verify_integrity,
            direction: args.direction,
        }),
    });
    wire::send_message(&mut ctrl_sock, &hello)?;

    // waits for server answer
    let session = match wire::read_message(&mut ctrl_sock)? {
        Message::SessionStart(s) => s,
        Message::Error(e) => bail_error!("ctrl", "server declined session establishment : {e}"),
        other => bail_error!("ctrl", "unknown error from server : {other:?}"),
    };

    // run the benchmark based on direction
    let (upload_stats, _download_stats): BenchmarkResult = match args.direction {
        Direction::Default => {
            let stats = stream::run_tcp_benchmark(
                args.server,
                session.data_ports[0],
                args.n_streams,
                session.seed,
                args.duration,
            )?;
            (Some(stats), None)
        }
        Direction::Reverse | Direction::Both | Direction::Bidirectional => {
            // TODO: implement download, both, bidirectional
            bail_error!("aw", "direction {:?} not yet implemented", args.direction);
        }
    };

    // server statistics retrieval
    let server_stats = match wire::read_message(&mut ctrl_sock)? {
        Message::SessionStats(SessionStats::Benchmark { upload, download }) => (upload, download),
        Message::Error(e) => bail_error!("ctrl", "server error: {e}"),
        other => bail_error!("ctrl", "unexpected message: {other:?}"),
    };

    Ok((upload_stats, server_stats.0))
}

/// runs the TCP client, connects to a server, and benchmarks the wire
pub fn run(args: BenchmarkParameters) -> Result<()> {
    info!("ctrl", "connected to {}:{}", args.server, args.port);
    info!("ctrl", "direction: {}", args.direction.description());

    // runs benchmark and gets stats
    let (upload_stats, server_stats) = run_internal(args)?;

    info!("ctrl", "benchmark done");
    info!("ctrl", "session statistics received from the server");

    // result print
    if let (Some(client_up), Some(server_up)) = (&upload_stats, &server_stats) {
        print_results("sender (client)", client_up, true);
        print_results("receiver (server)", server_up, false);
    }

    Ok(())
}

/// runs benchmark without printing results (for qualify mode)
pub fn run_silent(args: BenchmarkParameters) -> Result<BenchmarkResult> {
    run_internal(args)
}
