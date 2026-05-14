//! TCP stream server module

use anyhow::Result;
use std::net::UdpSocket;
use std::time::Instant;

use crate::protocol::stats::UdpStreamStats;
use crate::server::udp::statistics::{StreamState, compute_stats};
use crate::warn;

/// receives UDP streams from client
pub fn receive_udp_streams(sock: &UdpSocket, n_streams: u16) -> Result<Vec<UdpStreamStats>> {
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
                    let timestamp_send = u64::from_be_bytes(buf[10..18].try_into()?);

                    if stream_id < streams.len() {
                        let stream = &mut streams[stream_id];

                        stream.packets_recv += 1;
                        stream.bytes_received += n as u64;

                        if stream.first_recv_ts.is_none() {
                            stream.first_recv_ts = Some(recv_ts);
                        }

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
