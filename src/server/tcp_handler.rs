//! TCP session handler

use anyhow::Result;
use std::io::{ErrorKind, Read};
use std::net::{IpAddr, SocketAddr, TcpListener, TcpStream};
use std::time::Instant;

use crate::protocol::messages::{
    Direction, Message, SessionStart, SessionStats, TcpBenchmarkConfig,
};
use crate::protocol::stats::TcpStreamStats;
use crate::protocol::wire;
use crate::server::ServerParameters;
use crate::utils::payload::{make_buffer, stream_seed};
use crate::utils::random::rand_u64;
use crate::utils::report::print_results;
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
    info!(
        "ctrl",
        "informed the client the session can start (id: {session_id})"
    );

    // handle based on direction
    let (upload_stats, download_stats): (Option<Vec<TcpStreamStats>>, Option<Vec<TcpStreamStats>>) =
        match config.direction {
            Direction::Default => {
                let stats = receive_upload_streams(
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
        print_results("receiver (server)", stats, false);
    }
    if let Some(ref stats) = download_stats {
        print_results("sender (server)", stats, true);
    }

    info!("ctrl", "session complete");

    Ok(())
}

/// receives upload streams (client -> server)
fn receive_upload_streams(
    data_listener: &TcpListener,
    n_streams: u16,
    seed: u64,
    verify: bool,
) -> Result<Vec<TcpStreamStats>> {
    let mut handles = Vec::with_capacity(n_streams as usize);

    // threads launch
    for _ in 0..n_streams {
        let (mut data_sock, client) = data_listener.accept()?;

        // reads client's stream ID
        let mut id_bytes = [0u8; 2];
        data_sock.read_exact(&mut id_bytes)?;
        let stream_id = u16::from_be_bytes(id_bytes);

        info!("data", "stream {stream_id} connected from {client}");

        let session_seed = seed;

        let handle = std::thread::spawn(move || -> Result<TcpStreamStats> {
            receive_data(stream_id, session_seed, verify, data_sock)
        });
        handles.push(handle);
    }

    warn!(
        "data",
        "all {} stream(s) connected, session in progress...", n_streams
    );

    // joins threads and collects stats
    let mut streams: Vec<TcpStreamStats> = Vec::with_capacity(handles.len());
    for handle in handles {
        match handle.join() {
            Ok(Ok(s)) => streams.push(s),
            Ok(Err(e)) => bail_error!("data", "stream failed: {e:#}"),
            Err(_) => bail_error!("data", "stream thread panicked"),
        }
    }

    streams.sort_by_key(|s| s.stream_id);
    Ok(streams)
}

/// reads received data until client FIN
fn receive_data(
    stream_id: u16,
    session_seed: u64,
    verify: bool,
    mut sock: TcpStream,
) -> Result<TcpStreamStats> {
    // receiving buffer
    let mut buf = vec![0u8; 256 * 1024];

    // buffer verification
    const MAX_VERIFY_BUFFER: usize = 1024 * 1024 * 1024; // 1 GB hard limit

    let mut received_data = if verify {
        // pre-allocate 1 GB to avoid initial reallocations
        let mut v = Vec::new();
        v.reserve_exact(MAX_VERIFY_BUFFER);
        Some(v)
    } else {
        None
    };

    // counters
    let mut first: Option<Instant> = None;
    let mut last = Instant::now();
    let mut bytes_received: u64 = 0;

    // receiving loop
    loop {
        match sock.read(&mut buf) {
            Ok(0) => break, // FIN received
            Ok(n) => {
                if first.is_none() {
                    first = Some(Instant::now());
                }
                last = Instant::now();

                bytes_received += n as u64;

                // store data up to 1GB limit
                if let Some(ref mut data) = received_data
                    && data.len() < MAX_VERIFY_BUFFER
                {
                    let remaining = MAX_VERIFY_BUFFER - data.len();
                    let to_store = n.min(remaining);
                    data.extend_from_slice(&buf[..to_store]);
                }
            }
            Err(e) if e.kind() == ErrorKind::Interrupted => continue, // EINTR
            Err(e) => return Err(e.into()),
        }
    }

    let duration_ns = match first {
        Some(t0) => last.duration_since(t0).as_nanos() as u64,
        None => 0,
    };

    // post-session validation: verify only the stored data
    if let Some(received) = received_data {
        let verified_gb = received.len() as f64 / MAX_VERIFY_BUFFER as f64;
        let total_gb = bytes_received as f64 / MAX_VERIFY_BUFFER as f64;

        if received.len() < bytes_received as usize {
            warn!(
                "data",
                "stream {stream_id}: verifying first {verified_gb:.2} GiB of {total_gb:.2} GiB total..."
            );
        } else {
            warn!(
                "data",
                "stream {stream_id}: verifying {verified_gb:.2} GiB..."
            );
        }

        let expected = make_buffer(stream_seed(session_seed, stream_id));
        let expected_len = expected.len();

        // parallel verification by chunks
        use rayon::prelude::*;

        let verification_result: Result<()> = received
            .par_chunks(expected_len)
            .enumerate()
            .try_for_each(|(chunk_idx, chunk)| {
                let base_offset = chunk_idx * expected_len;
                // compare chunk against expected pattern
                if chunk != &expected[..chunk.len()] {
                    // find exact mismatch byte
                    for i in 0..chunk.len() {
                        if chunk[i] != expected[i] {
                            bail_error!(
                                "data",
                                "stream {}: integrity check failed at byte {}. expected 0x{:02x}, got 0x{:02x}",
                                stream_id,
                                base_offset + i,
                                expected[i],
                                chunk[i]
                            );
                        }
                    }
                }
                Ok(())
            });

        verification_result?;
        info!("data", "stream {stream_id}: integrity check passed");
    }

    Ok(TcpStreamStats {
        stream_id,
        bytes_sent: 0,
        bytes_received,
        duration_ns,
    })
}
