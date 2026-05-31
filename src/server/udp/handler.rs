//! UDP session handler

use anyhow::Result;
use std::net::{IpAddr, SocketAddr, TcpStream, UdpSocket};
use std::sync::mpsc::Sender;

use crate::protocol::messages::{Message, SessionStart, SessionStats, UdpBenchmarkConfig};
use crate::protocol::wire::send_message;
use crate::server::udp::streams::receive_udp_streams;
use crate::server::{ServerParameters, ServerTuiEvent};
use crate::socket::so_rcvbuf::set_so_rcvbuf;
use crate::utils::format::bytes_formatting::human_bps;
use crate::utils::format::report::print_udp_results;
use crate::utils::system::random::rand_u64;
use crate::{info, info_noprefix, warn};

/// handles a UDP session
pub fn handle_udp_session(
    mut ctrl_sock: TcpStream,
    ctrl_client: SocketAddr,
    config: UdpBenchmarkConfig,
    params: &ServerParameters,
    tui_tx: Option<Sender<ServerTuiEvent>>,
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

    // socket receiving buffer size
    let target_bytes = params.udp_recv_buffer as usize;
    match set_so_rcvbuf(&data_udp_sock, target_bytes) {
        Ok(allocated_bytes) => {
            if allocated_bytes == target_bytes {
                info!(
                    "data",
                    "socket receive buffer set to {} KB",
                    allocated_bytes / 1024
                );
            } else {
                warn!(
                    "data",
                    "socket receive buffer requested {} KB but OS allocated {} KB",
                    target_bytes / 1024,
                    allocated_bytes / 1024
                );
            }
        }
        Err(err) => {
            warn!("aw", "failed to configure SO_RCVBUF: {}", err);
        }
    }

    // session id & seed generation
    let session_id: u64 = rand_u64();
    let seed: u64 = rand_u64();

    // inform client session can start
    send_message(
        &mut ctrl_sock,
        &Message::SessionStart(SessionStart {
            session_id,
            seed,
            data_ports: vec![data_udp_port],
        }),
    )?;

    // receive UDP packets
    let stats = receive_udp_streams(&data_udp_sock, config.n_streams)?;

    info!("ctrl", "session complete");

    // send stats back to client
    send_message(
        &mut ctrl_sock,
        &Message::SessionStats(SessionStats::UdpBenchmark {
            upload: Some(stats.clone()),
            download: None,
        }),
    )?;
    info!("ctrl", "session statistics sent to the client");

    // print results server-side
    print_udp_results("receiver (server)", &stats, false);
    if let Some(ref tx) = tui_tx {
        let _ = tx.send(ServerTuiEvent::UdpSessionResult(stats.clone()));
    }

    info_noprefix!("");
    info!("ctrl", "session complete");

    Ok(())
}
