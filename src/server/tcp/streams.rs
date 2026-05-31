//! TCP stream server module

use anyhow::Result;
use std::io::{ErrorKind, Read};
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::time::Instant;

use crate::protocol::stats::TcpStreamStats;
use crate::server::tcp::verify;
use crate::socket::tcp_maxseg::get_tcp_maxseg;
use crate::{bail_error, info};

/// receives TCP streams (client -> server)
pub fn receive_tcp_streams(
    data_listener: &TcpListener,
    n_streams: u16,
    seed: u64,
    verify: Option<u64>,
) -> Result<Vec<TcpStreamStats>> {
    let mut handles = Vec::with_capacity(n_streams as usize);

    // threads launch
    for _ in 0..n_streams {
        let (mut data_sock, client) = data_listener.accept()?;

        // reads client's stream ID
        let mut id_bytes = [0u8; 2];
        data_sock.read_exact(&mut id_bytes)?;
        let stream_id = u16::from_be_bytes(id_bytes);

        // TCP stream info
        info!("data", "stream {stream_id} connected from {client}");

        // TCP_MAXSEG info
        let mss = get_tcp_maxseg(&data_sock)?;
        info!("data", "MSS = {mss}");

        let session_seed = seed;

        let handle = thread::spawn(move || -> Result<TcpStreamStats> {
            receive_tcp_stream(stream_id, session_seed, verify, data_sock)
        });
        handles.push(handle);
    }

    info!(
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

/// received TCP stream from client until client FIN
pub fn receive_tcp_stream(
    stream_id: u16,
    session_seed: u64,
    verify: Option<u64>,
    mut sock: TcpStream,
) -> Result<TcpStreamStats> {
    // receiving buffer
    let mut buf = vec![0u8; 256 * 1024];

    let mut received_data = verify.map(|verify_size| {
        let mut v = Vec::new();
        v.reserve_exact(verify_size as usize);
        v
    });

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

                // stores data for --verify
                if let (Some(data), Some(verify_size)) = (&mut received_data, verify)
                    && data.len() < verify_size as usize
                {
                    let remaining = verify_size as usize - data.len();
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
        verify::verify_received_data(stream_id, session_seed, bytes_received, received)?;
    }

    Ok(TcpStreamStats {
        stream_id,
        bytes_sent: 0,
        bytes_received,
        duration_ns,
    })
}
