//! aether-wire client

use anyhow::{Result, bail};
use std::io::{ErrorKind, Write};
use std::net::{IpAddr, Shutdown, SocketAddr, TcpStream};
use std::time::Instant;

use crate::cli::ClientArgs;
use crate::payload;
use crate::proto::{Hello, Message, PROTO_VERSION, SessionStats};
use crate::utils::{human_bps, human_bytes};
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

    // payload build
    let stream_id: u32 = 0; // will change when multiple-stream tests will come
    let buf = payload::make_buffer(payload::stream_seed(session.seed, stream_id));

    // data channel session establishment
    let data_addr = SocketAddr::new(IpAddr::V4(args.server), session.data_port);
    let mut data_sock = TcpStream::connect(data_addr)?;
    println!("[data] connected to {data_addr}, benchmark in progress...");

    // counters
    let start = Instant::now();
    let deadline = start + args.time;
    let mut bytes_sent: u64 = 0;

    // buffer cursor
    let mut cursor: usize = 0;

    // send loop
    while Instant::now() < deadline {
        let slice = &buf[cursor..];
        match data_sock.write(slice) {
            Ok(0) => break,
            Ok(n) => {
                bytes_sent += n as u64;
                cursor += n;
                if cursor >= buf.len() {
                    cursor = 0;
                }
            }
            Err(e) if e.kind() == ErrorKind::Interrupted => continue, // EINTR
            Err(e) => return Err(e.into()),
        }
    }

    let elapsed = start.elapsed();

    // ends data channel session (sends FIN)
    data_sock.shutdown(Shutdown::Write)?;
    println!("[data] done");

    // server statistics retrieval
    let server_stats: SessionStats = match wire::read_message(&mut ctrl_sock)? {
        Message::SessionStats(s) => s,
        Message::Error(e) => bail!("server error at the end of the test: {e}"),
        other => bail!("unexpected message at the end: {other:?}"),
    };
    println!("[ctrl] session statistics received from the server");

    // client statistics processing
    let client_secs = elapsed.as_secs_f64();
    let sent_bitrate = if client_secs > 0.0 {
        (bytes_sent as f64) * 8.0 / client_secs
    } else {
        0.0
    };

    // server statistics processing
    let server_secs = server_stats.duration_ns as f64 / 1_000_000_000.0;
    let recv_bitrate = if server_secs > 0.0 {
        (server_stats.bytes_received as f64) * 8.0 / server_secs
    } else {
        0.0
    };

    // result print
    println!("======== sender (client) ========");
    println!("duration : {client_secs:.2}s");
    println!(
        "sent     : {} ({} bytes)",
        human_bytes(bytes_sent),
        bytes_sent
    );
    println!("bitrate  : {}", human_bps(sent_bitrate));

    println!("======= receiver (server) =======");
    println!("duration : {server_secs:.2}s");
    println!(
        "received : {} ({} bytes)",
        human_bytes(server_stats.bytes_received),
        server_stats.bytes_received
    );
    println!("bitrate  : {}", human_bps(recv_bitrate));

    Ok(())
}
