//! aether-wire server

use anyhow::{Result, bail};
use std::io::{ErrorKind, Read};
use std::net::{IpAddr, SocketAddr, TcpListener, TcpStream};
use std::time::Instant;

use crate::cli::ServeArgs;
use crate::proto::{Hello, Message, PROTO_VERSION, SessionStart, SessionStats};
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
        "[ctrl] hello received, requested duration : {}s",
        hello.duration_secs
    );

    // data channel session establishment
    let data_listener = TcpListener::bind(SocketAddr::new(IpAddr::V4(args.bind), 0))?;
    let data_port = data_listener.local_addr()?.port();
    println!("[data] listening on port {data_port}");

    // session id & seed generation (future use)
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

    // waits for a client
    let (mut data_sock, data_client) = data_listener.accept()?;
    println!("[data] established connection from {data_client}, benchmark in progress...");

    // starts receiving data
    let (bytes_received, duration_ns) = receive_data(&mut data_sock)?;
    println!("[data] done");

    // sends server statistics
    let stats = SessionStats {
        bytes_received,
        duration_ns,
    };
    wire::send_message(&mut ctrl_sock, &Message::SessionStats(stats))?;
    println!("[ctrl] session statistics sent to the client");

    // server statistics processing
    let secs = duration_ns as f64 / 1_000_000_000.0;
    let bitrate = if secs > 0.0 {
        (bytes_received as f64) * 8.0 / secs
    } else {
        0.0
    };

    // result print
    println!("======= receiver (server) =======");
    println!("duration : {secs:.2}s");
    println!(
        "sent     : {} ({} bytes)",
        human_bytes(bytes_received),
        bytes_received
    );
    println!("bitrate  : {}", human_bps(bitrate));

    Ok(())
}

/// reads received data until client FIN
fn receive_data(sock: &mut TcpStream) -> Result<(u64, u64)> {
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

    Ok((bytes, duration_ns))
}
