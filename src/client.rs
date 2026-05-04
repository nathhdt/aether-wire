//! aether-wire client

use anyhow::{Result, bail};
use std::io::{ErrorKind, Write};
use std::net::{IpAddr, Shutdown, SocketAddr, TcpStream};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Barrier};
use std::time::Instant;

use crate::cli::ClientArgs;
use crate::payload;
use crate::proto::{Hello, Message, PROTO_VERSION, SessionStats, StreamStats};
use crate::utils::{print_results};
use crate::wire;

/// runs the client, connects to a server, and benchmarks the wire
pub fn run(args: ClientArgs) -> Result<()> {
    // control channel session establishment
    let ctrl_addr = SocketAddr::new(IpAddr::V4(args.server), args.port);
    let mut ctrl_sock = TcpStream::connect(ctrl_addr)?;
    println!("[ctrl] connected to {ctrl_addr}");

    // hello message
    let hello = Message::Hello(Hello {
        version: PROTO_VERSION,
        duration_secs: args.time.as_secs(),
        n_streams: args.n_streams,
    });
    wire::send_message(&mut ctrl_sock, &hello)?;
    println!("[ctrl] hello message sent");

    // waits for server answer
    let session = match wire::read_message(&mut ctrl_sock)? {
        Message::SessionStart(s) => s,
        Message::Error(e) => bail!("[ctrl] server declined session establishment : {e}"),
        other => bail!("[ctrl] unknown error from server : {other:?}"),
    };
    println!(
        "[ctrl] session {} - data port : {} - seed : {}",
        session.session_id, session.data_port, session.seed
    );

    // sets up thread elements for multi-stream benchmark
    let barrier = Arc::new(Barrier::new(args.n_streams as usize + 1));
    let stop = Arc::new(AtomicBool::new(false));
    let mut handles = Vec::with_capacity(args.n_streams as usize);

    // prepares socket specifications
    let data_addr = SocketAddr::new(IpAddr::V4(args.server), session.data_port);

    // threads launch
    for stream_id in 0..args.n_streams {
        let barrier = Arc::clone(&barrier);
        let stop = Arc::clone(&stop);

        // payload build
        let buf = payload::make_buffer(payload::stream_seed(session.seed, stream_id));

        // thread spawn
        let handle = std::thread::spawn(move || -> Result<StreamStats> {
            // data channel session establishment
            let mut data_sock = TcpStream::connect(data_addr)?;
            println!("[data] connected to {data_addr}");

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
            println!("[data] done ({data_addr})");

            Ok(StreamStats {
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
    println!("[data] all streams connected, benchmark in progress...");

    // waits for benchmark
    std::thread::sleep(args.time);

    // signals end of benchmark for all threads
    stop.store(true, Ordering::Relaxed);

    // gets statistics from threads
    let mut client_stats: Vec<StreamStats> = Vec::with_capacity(handles.len());
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

    println!("[data] benchmark done ({:.2}s)", time_elapsed.as_secs_f64());

    // server statistics retrieval
    let server_stats: SessionStats = match wire::read_message(&mut ctrl_sock)? {
        Message::SessionStats(s) => s,
        Message::Error(e) => bail!("server error: {e}"),
        other => bail!("unexpected message: {other:?}"),
    };
    println!("[ctrl] session statistics received from the server");

    // result print
    print_results("sender (client)", &client_stats, true);
    print_results("receiver (server)", &server_stats.streams, false);

    Ok(())
}
