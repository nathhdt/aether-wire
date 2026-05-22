//! TCP session handler

use anyhow::Result;
use std::net::{IpAddr, SocketAddr, TcpListener, TcpStream};

use crate::protocol::messages::{
    Direction, Message, SessionStart, SessionStats, TcpBenchmarkConfig,
};
use crate::protocol::stats::TcpStreamStats;
use crate::protocol::wire;
use crate::server::ServerParameters;
use crate::server::tcp::streams;
use crate::utils::format::report::print_tcp_results;
use crate::utils::random::rand_u64;
use crate::{bail_error, info, warn};

/// handles a TCP session
pub fn handle_tcp_session(
    mut ctrl_sock: TcpStream,
    ctrl_client: SocketAddr,
    config: TcpBenchmarkConfig,
    params: &ServerParameters,
) -> Result<()> {
    info!(
        "ctrl",
        "client {} asked for a TCP session ({} stream(s), {}s, direction: {})",
        ctrl_client,
        config.n_streams,
        config.duration_secs,
        config.direction.description()
    );

    if config.verify_integrity {
        warn!(
            "ctrl",
            "client requested server-side buffer verification (--verify)"
        );
    }

    // data channel session establishment
    let data_listener = TcpListener::bind(SocketAddr::new(IpAddr::V4(params.bind), 0))?;
    let data_port = data_listener.local_addr()?.port();
    info!("data", "TCP listening on port {data_port}");

    // session id & seed generation
    let session_id: u64 = rand_u64();
    let seed: u64 = rand_u64();

    // informs the client the session can start
    wire::send_message(
        &mut ctrl_sock,
        &Message::SessionStart(SessionStart {
            session_id,
            seed,
            data_ports: vec![data_port],
        }),
    )?;

    // handle based on direction
    let (upload_stats, download_stats): (Option<Vec<TcpStreamStats>>, Option<Vec<TcpStreamStats>>) =
        match config.direction {
            Direction::Default => {
                let stats = streams::receive_tcp_streams(
                    &data_listener,
                    config.n_streams,
                    seed,
                    config.verify_integrity,
                )?;
                (Some(stats), None)
            }
            Direction::Reverse | Direction::Both | Direction::Bidirectional => {
                // TODO: implement download, both, bidirectional
                let _ = wire::send_message(
                    &mut ctrl_sock,
                    &Message::Error(format!(
                        "direction {:?} not yet implemented",
                        config.direction
                    )),
                );
                bail_error!(
                    "ctrl",
                    "direction {:?} not yet implemented",
                    config.direction
                );
            }
        };

    info!("data", "session complete");

    // sends stats back to the client
    wire::send_message(
        &mut ctrl_sock,
        &Message::SessionStats(SessionStats::TcpBenchmark {
            upload: upload_stats.clone(),
            download: download_stats.clone(),
        }),
    )?;
    info!("ctrl", "session statistics sent to the client");

    // result print
    if let Some(ref stats) = upload_stats {
        print_tcp_results("receiver (server)", stats, false);
    }
    if let Some(ref stats) = download_stats {
        print_tcp_results("sender (server)", stats, true);
    }

    info!("ctrl", "session complete");

    Ok(())
}
