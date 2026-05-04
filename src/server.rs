//! aether-wire server

use anyhow::{Result, bail};
use std::io::{ErrorKind, Read};
use std::net::{IpAddr, SocketAddr, TcpListener, TcpStream};
use std::time::Instant;

use crate::cli::ServeArgs;
use crate::proto::{Hello, Message, PROTO_VERSION, SessionStart, SessionStats, StreamStats};
use crate::utils::{human_bps, human_bytes, rand_u64};
use crate::wire;

/// runs the server, listens for a connection, and benchmarks the wire
pub fn run(args: ServeArgs) -> Result<()> {
    // listens to the control channel session port
    let ctrl_addr = SocketAddr::new(IpAddr::V4(args.bind), args.port);
    let ctrl_listener = TcpListener::bind(ctrl_addr)?;
    println!("[ctrl] server listening on {ctrl_addr}");

    // accepts only one session
    let (mut ctrl_sock, ctrl_client) = ctrl_listener.accept()?;
    println!("[ctrl] accepted connection from {ctrl_client}");

    // reads hello message
    let hello: Hello = match wire::read_message(&mut ctrl_sock)? {
        Message::Hello(h) => h,
        other => {
            // informs client that hello message is expected
            let _ = wire::send_message(
                &mut ctrl_sock,
                &Message::Error("expected hello message".into()),
            );
            bail!("unexpected first message : {other:?}");
        }
    };

    // checks protocol version
    if hello.version != PROTO_VERSION {
        let incompatible_version_msg = format!(
            "incompatible version : client={}, server={}",
            hello.version, PROTO_VERSION
        );
        let _ = wire::send_message(
            &mut ctrl_sock,
            &Message::Error(incompatible_version_msg.clone()),
        );
        bail!("[ctrl] {incompatible_version_msg}");
    }

    println!(
        "[ctrl] hello received - {} stream(s), {}s",
        hello.n_streams, hello.duration_secs
    );

    // data channel session establishment
    let data_listener = TcpListener::bind(SocketAddr::new(IpAddr::V4(args.bind), 0))?;
    let data_port = data_listener.local_addr()?.port();
    println!("[data] listening on port {data_port}");

    // session id & seed generation
    let session_id: u64 = rand_u64();
    let seed: u64 = rand_u64();

    // informs the client the session can start
    wire::send_message(
        &mut ctrl_sock,
        &Message::SessionStart(SessionStart {
            session_id,
            seed,
            data_port,
        }),
    )?;
    println!("[ctrl] informed the client the session (id: {session_id}) can start");

    // sets up threads for multi-stream benchmark
    let mut handles = Vec::with_capacity(hello.n_streams as usize);

    // threads launch
    for stream_id in 0..hello.n_streams {
        let (mut data_sock, client) = data_listener.accept()?;
        println!("[data] stream {stream_id} connected from {client}");

        let handle = std::thread::spawn(move || -> Result<StreamStats> {
            receive_data(stream_id, &mut data_sock)
        });
        handles.push(handle);
    }

    println!(
        "[data] all {} stream(s) connected, benchmark in progress...",
        hello.n_streams
    );

    // joins threads and collects stats
    let mut streams: Vec<StreamStats> = Vec::with_capacity(handles.len());
    for handle in handles {
        match handle.join() {
            Ok(Ok(s)) => streams.push(s),
            Ok(Err(e)) => bail!("[data] stream failed: {e:#}"),
            Err(_) => bail!("[data] stream thread panicked"),
        }
    }

    streams.sort_by_key(|s| s.stream_id);
    println!("[data] all streams done");

    // sends stats back to the client
    wire::send_message(
        &mut ctrl_sock,
        &Message::SessionStats(SessionStats {
            streams: streams.clone(),
        }),
    )?;
    println!("[ctrl] session statistics sent to the client");

    // result print
    println!();
    println!("======= receiver (server) =======");

    let mut total_bytes: u64 = 0;
    let mut max_ns: u64 = 0;

    for s in &streams {
        let secs = s.duration_ns as f64 / 1_000_000_000.0;
        let bitrate = if secs > 0.0 {
            (s.bytes_received as f64) * 8.0 / secs
        } else {
            0.0
        };
        println!(
            "  stream {:>2} — received {} — {}",
            s.stream_id,
            human_bytes(s.bytes_received),
            human_bps(bitrate),
        );
        total_bytes += s.bytes_received;
        if s.duration_ns > max_ns {
            max_ns = s.duration_ns;
        }
    }

    if streams.len() > 1 {
        let secs = max_ns as f64 / 1_000_000_000.0;
        let bps = if secs > 0.0 {
            (total_bytes as f64) * 8.0 / secs
        } else {
            0.0
        };
        println!("  ────────────────────────────────────────");
        println!(
            "  total    — received {} — {}",
            human_bytes(total_bytes),
            human_bps(bps)
        );
    }

    Ok(())
}

/// reads received data until client FIN
fn receive_data(stream_id: u16, sock: &mut TcpStream) -> Result<StreamStats> {
    let mut buf = vec![0u8; 256 * 1024];
    let mut bytes: u64 = 0;

    // timer
    let mut first: Option<Instant> = None;
    let mut last = Instant::now();

    // receiving loop
    loop {
        match sock.read(&mut buf) {
            Ok(0) => break, // FIN received
            Ok(n) => {
                if first.is_none() {
                    first = Some(Instant::now());
                }
                last = Instant::now();
                bytes += n as u64;
            }
            Err(e) if e.kind() == ErrorKind::Interrupted => continue, // EINTR
            Err(e) => return Err(e.into()),
        }
    }

    let duration_ns = match first {
        Some(t0) => last.duration_since(t0).as_nanos() as u64,
        None => 0,
    };

    Ok(StreamStats {
        stream_id,
        bytes_sent: 0,
        bytes_received: bytes,
        duration_ns,
    })
}
