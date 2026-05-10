//! UDP session handler

use anyhow::Result;
use std::net::{IpAddr, SocketAddr, TcpStream, UdpSocket};
use std::time::Instant;

use crate::protocol::messages::{Message, SessionStart, UdpBenchmarkConfig};
use crate::protocol::stats::UdpStreamStats;
use crate::protocol::wire;
use crate::server::ServerParameters;
use crate::utils::format::human_bps;
use crate::utils::random::rand_u64;
use crate::{info, warn};

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
    info!(
        "ctrl",
        "informed the client the session can start (id: {session_id})"
    );

    // receive UDP packets
    let _stats = receive_udp_stream(&data_udp_sock, config.n_streams)?;

    info!("ctrl", "session complete");

    // TODO: send stats back to client
    // TODO: print results

    Ok(())
}

/// receives UDP packets from client
fn receive_udp_stream(sock: &UdpSocket, n_streams: u16) -> Result<Vec<UdpStreamStats>> {
    let mut buf = vec![0u8; 65536]; // max UDP datagram size

    let mut stats: Vec<UdpStreamStats> = (0..n_streams)
        .map(|id| UdpStreamStats {
            stream_id: id,
            ..Default::default()
        })
        .collect();

    let mut first: Option<Instant> = None;
    let mut last = Instant::now();

    warn!("data", "waiting for UDP packets...");

    loop {
        match sock.recv_from(&mut buf) {
            Ok((n, src)) => {
                if first.is_none() {
                    first = Some(Instant::now());
                    info!("data", "first packet received from {src}");
                }
                last = Instant::now();

                // TODO: parse packet header to get stream_id, seq_num, timestamp
                // For now, just count bytes on stream 0
                if !stats.is_empty() {
                    stats[0].bytes_received += n as u64;
                    stats[0].packets_recv += 1;
                }

                // Simple timeout detection: if no packet for 2s, assume done
                if let Some(t0) = first
                    && last.duration_since(t0).as_secs() > 2
                    && Instant::now().duration_since(last).as_secs() > 2
                {
                    break;
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                // timeout, check if we're done
                if let Some(t0) = first
                    && Instant::now().duration_since(t0).as_secs() > 10
                {
                    break;
                }
            }
            Err(e) => return Err(e.into()),
        }
    }

    let duration_ns = match first {
        Some(t0) => last.duration_since(t0).as_nanos() as u64,
        None => 0,
    };

    for stat in &mut stats {
        stat.duration_ns = duration_ns;
    }

    info!(
        "data",
        "received {} packets, {} bytes total",
        stats.iter().map(|s| s.packets_recv).sum::<u64>(),
        stats.iter().map(|s| s.bytes_received).sum::<u64>()
    );

    Ok(stats)
}
