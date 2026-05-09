//! TCP stream management for benchmark mode

use anyhow::{Result, bail};
use std::io::{ErrorKind, Write};
use std::net::{IpAddr, Shutdown, SocketAddr, TcpStream};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Barrier};
use std::time::{Duration, Instant};

use crate::protocol::stats::TcpStreamStats;
use crate::transport::tcp::maxseg::get_tcp_maxseg;
use crate::utils::payload;

/// runs a multi-stream TCP benchmark
pub fn run_tcp_benchmark(
    server: std::net::Ipv4Addr,
    port: u16,
    n_streams: u16,
    session_seed: u64,
    duration: Duration,
) -> Result<Vec<TcpStreamStats>> {
    // sets up thread elements for multi-stream benchmark
    let barrier = Arc::new(Barrier::new(n_streams as usize + 1));
    let stop = Arc::new(AtomicBool::new(false));
    let mut handles = Vec::with_capacity(n_streams as usize);

    // prepares socket specifications
    let data_addr = SocketAddr::new(IpAddr::V4(server), port);

    // threads launch
    for stream_id in 0..n_streams {
        let barrier = Arc::clone(&barrier);
        let stop = Arc::clone(&stop);

        // payload build
        let buf = payload::make_buffer(payload::stream_seed(session_seed, stream_id));

        // thread spawn
        let handle = std::thread::spawn(move || -> Result<TcpStreamStats> {
            run_single_stream(stream_id, data_addr, buf, barrier, stop)
        });

        handles.push(handle);
    }

    // waits for all threads to be ready
    barrier.wait();

    println!(
        "[data] all {} stream(s) connected, benchmark in progress...",
        n_streams
    );

    // waits for benchmark
    std::thread::sleep(duration);

    // signals end of benchmark for all threads
    stop.store(true, Ordering::Relaxed);

    println!("[data] all streams done");

    // gets statistics from threads
    let mut client_stats: Vec<TcpStreamStats> = Vec::with_capacity(handles.len());
    for handle in handles {
        match handle.join() {
            Ok(Ok(s)) => client_stats.push(s),
            Ok(Err(e)) => bail!("[data] stream failed: {e:#}"),
            Err(_) => bail!("[data] stream thread panicked"),
        }
    }

    client_stats.sort_by_key(|s| s.stream_id);

    Ok(client_stats)
}

/// runs a single TCP stream
fn run_single_stream(
    stream_id: u16,
    data_addr: SocketAddr,
    buf: Vec<u8>,
    barrier: Arc<Barrier>,
    stop: Arc<AtomicBool>,
) -> Result<TcpStreamStats> {
    // data channel session establishment
    let mut data_sock = TcpStream::connect(data_addr)?;

    // TCP_MAXSEG info
    let mss = get_tcp_maxseg(&data_sock)?;

    println!("[data] stream {stream_id} connected to {data_addr}, TCP_MAXSEG = {mss}");

    // stream_id send through the wire before any benchmark starts
    data_sock.write_all(&stream_id.to_be_bytes())?;

    // counters
    let mut b_sent: u64 = 0;
    let mut cursor: usize = 0;

    // waits for all threads to be connected
    barrier.wait();

    // timer launch
    let start = Instant::now();

    // send loop
    while !stop.load(Ordering::Relaxed) {
        let slice = &buf[cursor..];
        match data_sock.write(slice) {
            Ok(0) => break,
            Ok(n) => {
                b_sent += n as u64;
                cursor += n;
                if cursor >= buf.len() {
                    cursor = 0;
                }
            }
            Err(e) if e.kind() == ErrorKind::Interrupted => continue, // EINTR
            Err(e) => bail!("stream {}: {}", stream_id, e),
        }
    }

    let duration_ns = start.elapsed().as_nanos() as u64;

    // ends data channel session (sends FIN)
    data_sock.shutdown(Shutdown::Write)?;

    Ok(TcpStreamStats {
        stream_id,
        bytes_sent: b_sent,
        bytes_received: 0,
        duration_ns,
    })
}
