//! aether-wire benchmark mode client endpoints

use anyhow::Result;
use std::net::{IpAddr, SocketAddr, TcpStream};

use crate::client::benchmark::{direction::Direction, tcp_stream};
use crate::protocol::messages::{
    Hello, Message, PROTO_VERSION, SessionStats, SessionType, TcpBenchmarkConfig,
    UdpBenchmarkConfig,
};
use crate::protocol::stats::{TcpStreamStats, UdpStreamStats};
use crate::protocol::wire;
use crate::utils::format::human_bps;
use crate::utils::report::print_results;
use crate::{bail_error, info};

/// client TCP benchmark arguments structure
pub struct TcpBenchmarkParameters {
    pub server: std::net::Ipv4Addr,
    pub port: u16,
    pub duration: std::time::Duration,
    pub n_streams: u16,
    pub verify_integrity: bool,
    pub direction: Direction,
}

/// TCP benchmark result containing stream statistics
pub type TcpBenchmarkResult = (Option<Vec<TcpStreamStats>>, Option<Vec<TcpStreamStats>>);

/// runs the TCP benchmark client, connects to a server, and benchmarks the wire
pub fn run_tcp(args: TcpBenchmarkParameters) -> Result<()> {
    info!("ctrl", "connected to {}:{}", args.server, args.port);
    info!("ctrl", "direction: {}", args.direction.description());

    // runs benchmark and gets stats
    let (upload_stats, server_stats) = run_tcp_internal(args)?;

    info!("ctrl", "TCP session done");
    info!("ctrl", "session statistics received from the server");

    // result print
    if let (Some(client_up), Some(server_up)) = (&upload_stats, &server_stats) {
        print_results("sender (client)", client_up, true);
        print_results("receiver (server)", server_up, false);
    }

    Ok(())
}

/// runs TCP benchmark without printing results
pub fn run_tcp_silent(args: TcpBenchmarkParameters) -> Result<TcpBenchmarkResult> {
    run_tcp_internal(args)
}

/// internal TCP benchmark execution, returns stats without printing
fn run_tcp_internal(args: TcpBenchmarkParameters) -> Result<TcpBenchmarkResult> {
    // control channel session establishment
    let ctrl_addr = SocketAddr::new(IpAddr::V4(args.server), args.port);
    let mut ctrl_sock = TcpStream::connect(ctrl_addr)?;

    // hello message with protocol
    let hello = Message::Hello(Hello {
        version: PROTO_VERSION,
        session_type: SessionType::TcpBenchmark(TcpBenchmarkConfig {
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
    let (upload_stats, _download_stats): TcpBenchmarkResult = match args.direction {
        Direction::Default => {
            let stats = tcp_stream::run_tcp_benchmark(
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
        Message::SessionStats(SessionStats::TcpBenchmark { upload, download }) => {
            (upload, download)
        }
        Message::Error(e) => bail_error!("ctrl", "server error: {e}"),
        other => bail_error!("ctrl", "unexpected message: {other:?}"),
    };

    Ok((upload_stats, server_stats.0))
}

/// client UDP benchmark arguments structure
pub struct UdpBenchmarkParameters {
    pub server: std::net::Ipv4Addr,
    pub port: u16,
    pub duration: std::time::Duration,
    pub n_streams: u16,
    pub bandwidth: u64,
    pub payload_size: u16,
}

/// UDP benchmark result containing stream statistics
pub type UdpBenchmarkResult = (Option<Vec<UdpStreamStats>>, Option<Vec<UdpStreamStats>>);

/// runs the UDP benchmark client (placeholder)
pub fn run_udp(args: UdpBenchmarkParameters) -> Result<()> {
    info!("ctrl", "connected to {}:{}", args.server, args.port);
    info!(
        "ctrl",
        "target bandwidth: {}, payload size: {} bytes",
        human_bps(args.bandwidth as f64),
        args.payload_size
    );

    // runs benchmark and gets stats
    let (_upload_stats, _server_stats) = run_udp_internal(args)?;

    info!("ctrl", "UDP session done");

    Ok(())
}

/// internal UDP benchmark execution, returns stats without printing
fn run_udp_internal(args: UdpBenchmarkParameters) -> Result<UdpBenchmarkResult> {
    // control channel session establishment
    let ctrl_addr = SocketAddr::new(IpAddr::V4(args.server), args.port);
    let mut ctrl_sock = TcpStream::connect(ctrl_addr)?;

    // hello message with protocol
    let hello = Message::Hello(Hello {
        version: PROTO_VERSION,
        session_type: SessionType::UdpBenchmark(UdpBenchmarkConfig {
            duration_secs: args.duration.as_secs(),
            n_streams: args.n_streams,
            bandwidth: args.bandwidth,
            payload_size: args.payload_size,
        }),
    });
    wire::send_message(&mut ctrl_sock, &hello)?;

    // waits for server answer
    let _session = match wire::read_message(&mut ctrl_sock)? {
        Message::SessionStart(s) => s,
        Message::Error(e) => bail_error!("ctrl", "server declined session establishment : {e}"),
        other => bail_error!("ctrl", "unknown error from server : {other:?}"),
    };

    // TODO: run UDP benchmark
    // TODO: receive server stats

    anyhow::bail!("UDP benchmark not yet implemented")
}
