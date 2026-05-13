//! UDP stream management

use anyhow::Result;
use std::net::{IpAddr, SocketAddr, UdpSocket};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Barrier};
use std::time::{Duration, Instant};

use crate::protocol::stats::UdpStreamStats;
use crate::utils::payload;
use crate::{bail_error, info, warn};

/// UDP payload in-header aether-wire size
const HEADER_SIZE: usize = 18;

/// runs a multi-stream UDP benchmark
pub fn run_udp_benchmark(
    server: std::net::Ipv4Addr,
    port: u16,
    n_streams: u16,
    session_seed: u64,
    duration: Duration,
    bandwidth: u64,
    payload_size: u16,
) -> Result<Vec<UdpStreamStats>> {
    // sets up multi-stream elements
    let barrier = Arc::new(Barrier::new(n_streams as usize + 1));
    let stop = Arc::new(AtomicBool::new(false));
    let mut handles = Vec::with_capacity(n_streams as usize);

    // bandwidth per stream
    let bandwidth_per_stream = bandwidth / n_streams as u64;

    // server address
    let data_addr = SocketAddr::new(IpAddr::V4(server), port);

    // launch threads
    for stream_id in 0..n_streams {
        let barrier = Arc::clone(&barrier);
        let stop = Arc::clone(&stop);

        // payload buffer
        let buf = payload::make_buffer(payload::stream_seed(session_seed, stream_id));

        // thread spawn
        let handle = std::thread::spawn(move || -> Result<UdpStreamStats> {
            run_single_udp_stream(
                stream_id,
                data_addr,
                buf,
                barrier,
                stop,
                bandwidth_per_stream,
                payload_size,
            )
        });

        handles.push(handle);
    }

    // wait for all threads to be ready
    barrier.wait();

    warn!("data", "all {} UDP stream(s) ready, sending...", n_streams);

    // wait for benchmark duration
    std::thread::sleep(duration);

    // signal end of benchmark
    stop.store(true, Ordering::Relaxed);

    info!("data", "all UDP streams done");

    // collect stats from threads
    let mut client_stats: Vec<UdpStreamStats> = Vec::with_capacity(handles.len());
    for handle in handles {
        match handle.join() {
            Ok(Ok(s)) => client_stats.push(s),
            Ok(Err(e)) => bail_error!("data", "UDP stream failed: {e:#}"),
            Err(_) => bail_error!("data", "UDP stream thread panicked"),
        }
    }

    client_stats.sort_by_key(|s| s.stream_id);

    Ok(client_stats)
}

/// runs a single UDP stream
fn run_single_udp_stream(
    stream_id: u16,
    server_addr: SocketAddr,
    payload_buf: Vec<u8>,
    barrier: Arc<Barrier>,
    stop: Arc<AtomicBool>,
    bandwidth: u64,
    payload_size: u16,
) -> Result<UdpStreamStats> {
    // create UDP socket
    let sock = UdpSocket::bind("0.0.0.0:0")?;
    sock.connect(server_addr)?;
    info!("data", "UDP stream {stream_id} connected to {server_addr}");

    // calculate packet timing for bandwidth control
    let packet_size = HEADER_SIZE + payload_size as usize;
    let bits_per_packet = (packet_size * 8) as u64;

    if bandwidth < bits_per_packet {
        bail_error!(
            "data",
            "stream {stream_id}: bandwidth too low for packet size (need at least {} bits/s per stream)",
            bits_per_packet
        );
    }

    let packets_per_sec = bandwidth / bits_per_packet;
    let interval_ns = 1_000_000_000 / packets_per_sec;

    // prepare packet buffer
    let mut packet = vec![0u8; packet_size];

    // write stream_id
    packet[0..2].copy_from_slice(&stream_id.to_be_bytes());

    // counters
    let mut seq_num: u64 = 0;
    let mut bytes_sent: u64 = 0;
    let mut packets_sent: u64 = 0;

    // wait for all threads
    barrier.wait();

    let start = Instant::now();
    let mut next_send = start;

    while !stop.load(Ordering::Relaxed) {
        let now = Instant::now();

        if now >= next_send {
            // builds packet header
            let timestamp_ns = now.duration_since(start).as_nanos() as u64;

            // seq_num
            packet[2..10].copy_from_slice(&seq_num.to_be_bytes());
            // timestamp_ns
            packet[10..18].copy_from_slice(&timestamp_ns.to_be_bytes());

            // payload: cycle through buffer
            let payload_offset = (seq_num as usize * payload_size as usize) % payload_buf.len();
            let mut payload_written = 0;

            while payload_written < payload_size as usize {
                let src_start = (payload_offset + payload_written) % payload_buf.len();
                let remaining = payload_size as usize - payload_written;
                let to_copy = remaining.min(payload_buf.len() - src_start);

                packet[HEADER_SIZE + payload_written..HEADER_SIZE + payload_written + to_copy]
                    .copy_from_slice(&payload_buf[src_start..src_start + to_copy]);

                payload_written += to_copy;
            }

            // send packet
            match sock.send(&packet) {
                Ok(n) => {
                    bytes_sent += n as u64;
                    packets_sent += 1;
                    seq_num += 1;
                }
                Err(e) => {
                    warn!("data", "stream {stream_id}: send error: {e}");
                }
            }

            // schedule next send
            next_send += Duration::from_nanos(interval_ns);

            // drift correction: if we're falling behind, catch up
            if next_send < now {
                next_send = now;
            }
        } else {
            // spin wait for maximum precision
            std::hint::spin_loop();
        }
    }

    let duration_ns = start.elapsed().as_nanos() as u64;

    Ok(UdpStreamStats {
        stream_id,
        bytes_sent,
        bytes_received: 0,
        packets_sent,
        packets_recv: 0,
        packets_lost: 0,
        packets_out_of_order: 0,
        packets_duplicate: 0,
        jitter_mean_ms: 0,
        jitter_median_ms: 0,
        jitter_stddev_ms: 0,
        duration_ns,
    })
}
