//! UDP session handler

use anyhow::Result;
use std::net::{IpAddr, SocketAddr, TcpStream, UdpSocket};
use std::time::Instant;

use crate::protocol::messages::{Message, SessionStart, SessionStats, UdpBenchmarkConfig};
use crate::protocol::stats::UdpStreamStats;
use crate::protocol::wire;
use crate::server::ServerParameters;
use crate::utils::format::human_bps;
use crate::utils::random::rand_u64;
use crate::utils::report::print_udp_results;
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

    // receive UDP packets
    let stats = receive_udp_stream(&data_udp_sock, config.n_streams)?;

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

/// UDP stream runtime statistics
#[derive(Default)]
struct StreamState {
    packets_recv: u64,
    bytes_received: u64,

    last_send_ts: Option<u64>,
    last_recv_ts: Option<u64>,

    // RFC3550 interarrival jitter estimator
    jitter_ns: f64,

    duration_ns: u64,
}

/// receives UDP stream from client
fn receive_udp_stream(sock: &UdpSocket, n_streams: u16) -> Result<Vec<UdpStreamStats>> {
    let mut buf = vec![0u8; 65536];

    // per-stream runtime state
    let mut streams: Vec<StreamState> = (0..n_streams).map(|_| StreamState::default()).collect();

    warn!("data", "waiting for UDP packets...");

    sock.set_read_timeout(Some(std::time::Duration::from_secs(4)))?;

    let start = Instant::now();

    // receiving loop
    loop {
        match sock.recv_from(&mut buf) {
            Ok((n, _)) => {
                if n >= 18 {
                    let recv_ts = start.elapsed().as_nanos() as u64;

                    let stream_id = ((buf[0] as usize) << 8) | (buf[1] as usize);
                    let timestamp_send = u64::from_be_bytes(buf[10..18].try_into().unwrap());

                    if stream_id < streams.len() {
                        let stream = &mut streams[stream_id];

                        stream.packets_recv += 1;
                        stream.bytes_received += n as u64;
                        stream.duration_ns = recv_ts;

                        // RFC3550 interarrival jitter
                        if let (Some(prev_send), Some(prev_recv)) =
                            (stream.last_send_ts, stream.last_recv_ts)
                        {
                            let send_delta = timestamp_send as i64 - prev_send as i64;
                            let recv_delta = recv_ts as i64 - prev_recv as i64;
                            let d = (recv_delta - send_delta).abs() as f64;

                            stream.jitter_ns += (d - stream.jitter_ns) / 16.0;
                        }

                        stream.last_send_ts = Some(timestamp_send);
                        stream.last_recv_ts = Some(recv_ts);
                    }
                }
            }
            Err(e)
                if e.kind() == std::io::ErrorKind::WouldBlock
                    || e.kind() == std::io::ErrorKind::TimedOut =>
            {
                break;
            }
            Err(e) => return Err(e.into()),
        }
    }

    // stats compute
    compute_stats(streams)
}

/// statistics compute for received UDP packets
fn compute_stats(streams: Vec<StreamState>) -> Result<Vec<UdpStreamStats>> {
    let mut stats = Vec::new();

    for (stream_id, stream) in streams.iter().enumerate() {
        stats.push(UdpStreamStats {
            stream_id: stream_id as u16,

            bytes_sent: 0,
            bytes_received: stream.bytes_received,

            packets_sent: 0,
            packets_recv: stream.packets_recv,

            packets_lost: 0,
            packets_out_of_order: 0,
            packets_duplicate: 0,

            // jitter to milliseconds
            jitter_rfc3550_ms: (stream.jitter_ns / 1_000_000.0) as u64,

            duration_ns: stream.duration_ns,
        });
    }

    Ok(stats)
}
