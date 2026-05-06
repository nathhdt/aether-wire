//! aether-wire TCP client

use anyhow::{Result, bail};
use std::io::{ErrorKind, Write};
use std::net::{IpAddr, Shutdown, SocketAddr, TcpStream};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Barrier};
use std::time::Instant;

use crate::cli::Ipv4TcpClientArgs;
use crate::payload;
use crate::proto::{Hello, Message, PROTO_VERSION, Protocol, SessionStats, TcpStreamStats};
use crate::tcp_utils::get_mss;
use crate::utils::print_results;
use crate::wire;

/// runs the TCP client, connects to a server, and benchmarks the wire
pub fn run(args: Ipv4TcpClientArgs) -> Result<()> {
    // control channel session establishment
    let ctrl_addr = SocketAddr::new(IpAddr::V4(args.server), args.port);
    let mut ctrl_sock = TcpStream::connect(ctrl_addr)?;
    println!("[ctrl] connected to {ctrl_addr}");

    // hello message
    let hello = Message::Hello(Hello {
        version: PROTO_VERSION,
        protocol: Protocol::Tcp,
        duration_secs: args.time.as_secs(),
        n_streams: args.n_streams,
        verify_integrity: args.verify,
    });
    wire::send_message(&mut ctrl_sock, &hello)?;

    // waits for server answer
    let session = match wire::read_message(&mut ctrl_sock)? {
        Message::SessionStart(s) => s,
        Message::Error(e) => bail!("[ctrl] server declined session establishment : {e}"),
        other => bail!("[ctrl] unknown error from server : {other:?}"),
    };
    println!(
        "[ctrl] session can start (id: {}, data port: {}, seed: {})",
        session.session_id, session.data_ports[0], session.seed
    );

    // sets up thread elements for multi-stream benchmark
    let barrier = Arc::new(Barrier::new(args.n_streams as usize + 1));
    let stop = Arc::new(AtomicBool::new(false));
    let mut handles = Vec::with_capacity(args.n_streams as usize);

    // prepares socket specifications
    let data_addr = SocketAddr::new(IpAddr::V4(args.server), session.data_ports[0]);

    // threads launch
    for stream_id in 0..args.n_streams {
        let barrier = Arc::clone(&barrier);
        let stop = Arc::clone(&stop);

        // payload build
        let buf = payload::make_buffer(payload::stream_seed(session.seed, stream_id));

        // thread spawn
        let handle = std::thread::spawn(move || -> Result<TcpStreamStats> {
            // data channel session establishment
            let mut data_sock = TcpStream::connect(data_addr)?;

            // reads session MSS
            let mss = get_mss(&data_sock)?;

            println!("[data] stream {stream_id} connected to {data_addr}, MSS = {mss}");

            // stream_id send through the wire - before any benchmark starts
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
        });

        handles.push(handle);
    }

    // waits for all threads to be ready
    barrier.wait();

    // timer launch
    let time_start = Instant::now();
    println!(
        "[data] all {} stream(s) connected, benchmark in progress...",
        args.n_streams
    );

    // waits for benchmark
    std::thread::sleep(args.time);

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

    // stops timer
    let time_elapsed = time_start.elapsed();

    client_stats.sort_by_key(|s| s.stream_id);

    println!("[ctrl] benchmark done ({:.2}s)", time_elapsed.as_secs_f64());

    // server statistics retrieval
    let server_stats: Vec<TcpStreamStats> = match wire::read_message(&mut ctrl_sock)? {
        Message::SessionStats(SessionStats::Tcp(s)) => s,
        Message::Error(e) => bail!("server error: {e}"),
        other => bail!("unexpected message: {other:?}"),
    };
    println!("[ctrl] session statistics received from the server");

    // result print
    print_results("sender (client)", &client_stats, true);
    print_results("receiver (server)", &server_stats, false);

    Ok(())
}
