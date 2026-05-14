//! UDP session handler

use anyhow::Result;
use std::net::{IpAddr, SocketAddr, TcpStream, UdpSocket};

use crate::info;
use crate::protocol::messages::{Message, SessionStart, SessionStats, UdpBenchmarkConfig};
use crate::protocol::wire;
use crate::server::ServerParameters;
use crate::server::udp::streams;
use crate::utils::format::human_bps;
use crate::utils::random::rand_u64;
use crate::utils::report::print_udp_results;

/// handles a UDP session
pub fn handle_udp_session(
    mut ctrl_sock: TcpStream,
    ctrl_client: SocketAddr,
    config: UdpBenchmarkConfig,
    params: &ServerParameters,
) -> Result<()> {
    info!(
        "ctrl",
        "client {} asked for a UDP session ({} stream(s), {}s, {})",
        ctrl_client,
        config.n_streams,
        config.duration_secs,
        human_bps(config.bandwidth as f64),
    );

    // UDP socket setup
    let data_udp_sock = UdpSocket::bind(SocketAddr::new(IpAddr::V4(params.bind), 0))?;
    let data_udp_port = data_udp_sock.local_addr()?.port();
    info!("data", "UDP listening on port {data_udp_port}");

    // session id & seed generation
    let session_id: u64 = rand_u64();
    let seed: u64 = rand_u64();

    // inform client session can start
    wire::send_message(
        &mut ctrl_sock,
        &Message::SessionStart(SessionStart {
            session_id,
            seed,
            data_ports: vec![data_udp_port],
        }),
    )?;

    // receive UDP packets
    let stats = streams::receive_udp_streams(&data_udp_sock, config.n_streams)?;

    info!("ctrl", "session complete");

    // send stats back to client
    wire::send_message(
        &mut ctrl_sock,
        &Message::SessionStats(SessionStats::UdpBenchmark {
            upload: Some(stats.clone()),
            download: None,
        }),
    )?;
    info!("ctrl", "session statistics sent to the client");

    // print results server-side
    print_udp_results("receiver (server)", &stats, false);

    info!("ctrl", "session complete");

    Ok(())
}
