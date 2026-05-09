//! aether-wire benchmark mode client

use anyhow::{Result, bail};
use std::net::{IpAddr, SocketAddr, TcpStream};

use crate::client::benchmark::stream;
use crate::protocol::messages::{
    BenchmarkConfig, Direction, Hello, Message, PROTO_VERSION, SessionStats, SessionType,
};
use crate::protocol::stats::TcpStreamStats;
use crate::protocol::wire;
use crate::utils::report::print_results;

/// client benchmark arguments structure
pub struct BenchmarkParameters {
    pub server: std::net::Ipv4Addr,
    pub port: u16,
    pub duration: std::time::Duration,
    pub n_streams: u16,
    pub verify_integrity: bool,
    pub direction: Direction,
}

/// runs the TCP client, connects to a server, and benchmarks the wire
pub fn run(args: BenchmarkParameters) -> Result<()> {
    // control channel session establishment
    let ctrl_addr = SocketAddr::new(IpAddr::V4(args.server), args.port);
    let mut ctrl_sock = TcpStream::connect(ctrl_addr)?;
    println!("[ctrl] connected to {ctrl_addr}");

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
        Message::Error(e) => bail!("[ctrl] server declined session establishment : {e}"),
        other => bail!("[ctrl] unknown error from server : {other:?}"),
    };
    println!(
        "[ctrl] session can start (id: {}, data port: {}, seed: {})",
        session.session_id, session.data_ports[0], session.seed
    );
    println!("[ctrl] direction: {}", args.direction.description());

    // run the benchmark based on direction
    let (upload_stats, download_stats): (Option<Vec<TcpStreamStats>>, Option<Vec<TcpStreamStats>>) =
        match args.direction {
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
                println!("[ctrl] direction {:?} not yet implemented", args.direction);
                return Ok(());
            }
        };

    println!("[ctrl] benchmark done");

    // server statistics retrieval
    let server_stats = match wire::read_message(&mut ctrl_sock)? {
        Message::SessionStats(SessionStats::Benchmark { upload, download }) => (upload, download),
        Message::Error(e) => bail!("server error: {e}"),
        other => bail!("unexpected message: {other:?}"),
    };
    println!("[ctrl] session statistics received from the server");

    // result print
    if let (Some(client_up), Some(server_up)) = (&upload_stats, &server_stats.0) {
        print_results("sender (client)", client_up, true);
        print_results("receiver (server)", server_up, false);
    }

    if let (Some(client_down), Some(server_down)) = (&download_stats, &server_stats.1) {
        print_results("receiver (client)", client_down, false);
        print_results("sender (server)", server_down, true);
    }

    Ok(())
}
