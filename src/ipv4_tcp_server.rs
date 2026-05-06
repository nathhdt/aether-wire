//! aether-wire TCP server

use anyhow::{Result, bail};
use std::io::{ErrorKind, Read};
use std::net::{IpAddr, SocketAddr, TcpListener, TcpStream};
use std::time::Instant;

use crate::cli::Ipv4TcpServerArgs;
use crate::proto::{Hello, Message, PROTO_VERSION, SessionStart, SessionStats, TcpStreamStats};
use crate::tcp_utils::get_mss;
use crate::utils::{print_results, rand_u64};
use crate::wire;

/// runs the TCP server, listens for a connection, and benchmarks the wire
pub fn run(args: Ipv4TcpServerArgs) -> Result<()> {
    // listens to the control channel session port
    let ctrl_addr = SocketAddr::new(IpAddr::V4(args.bind), args.port);
    let ctrl_listener = TcpListener::bind(ctrl_addr)?;
    println!("[ctrl] server listening on {ctrl_addr}");

    // server loop
    loop {
        println!("[ctrl] waiting for client...");

        // accepts only one session
        let (mut ctrl_sock, ctrl_client) = ctrl_listener.accept()?;

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

        // checks hello protocol version
        if hello.version != PROTO_VERSION {
            let msg = format!(
                "incompatible version : client={}, server={}",
                hello.version, PROTO_VERSION
            );

            let _ = wire::send_message(&mut ctrl_sock, &Message::Error(msg.clone()));
            bail!("[ctrl] {msg}");
        }

        println!(
            "[ctrl] client {} asked for a benchmark ({} stream(s), {}s)",
            ctrl_client, hello.n_streams, hello.duration_secs
        );

        if hello.verify_integrity {
            println!(
                "[ctrl] client requested server-side buffer verification, this may impact performance"
            );
        }

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
                data_ports: vec![data_port],
            }),
        )?;
        println!("[ctrl] informed the client the session can start (id: {session_id})");

        // sets up threads for multi-stream benchmark
        let mut handles = Vec::with_capacity(hello.n_streams as usize);

        // threads launch
        for _ in 0..hello.n_streams {
            let (mut data_sock, client) = data_listener.accept()?;

            // reads client's stream ID
            let mut id_bytes = [0u8; 2];
            data_sock.read_exact(&mut id_bytes)?;
            let stream_id = u16::from_be_bytes(id_bytes);

            // reads session MSS
            let mss = get_mss(&data_sock)?;

            println!("[data] stream {stream_id} connected from {client}, MSS = {mss}");

            let verify = hello.verify_integrity;
            let session_seed = seed;

            let handle = std::thread::spawn(move || -> Result<TcpStreamStats> {
                receive_data(stream_id, session_seed, verify, data_sock)
            });
            handles.push(handle);
        }

        println!(
            "[data] all {} stream(s) connected, benchmark in progress...",
            hello.n_streams
        );

        // joins threads and collects stats
        let mut streams: Vec<TcpStreamStats> = Vec::with_capacity(handles.len());
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
            &Message::SessionStats(SessionStats::Tcp(streams.clone())),
        )?;
        println!("[ctrl] session statistics sent to the client");

        // result print
        print_results("receiver (server)", &streams, false);

        println!("\n[ctrl] session complete");

        if args.once {
            println!("[ctrl] --once flag set, exiting");
            break;
        }
    }

    Ok(())
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
    let mut bytes: u64 = 0;

    // generates expected buffer (--verify)
    let expected_buffer = if verify {
        Some(crate::payload::make_buffer(crate::payload::stream_seed(
            session_seed,
            stream_id,
        )))
    } else {
        None
    };
    let mut cursor: usize = 0;

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

                // integrity verification if asked (--verify)
                if let Some(ref exp) = expected_buffer {
                    for (i, &received) in buf.iter().enumerate().take(n) {
                        let expected_byte = exp[cursor];

                        if received != expected_byte {
                            bail!(
                                "[data] stream {}: integrity check failed at byte {} (offset in buffer: {}): expected 0x{:02x}, got 0x{:02x}",
                                stream_id,
                                bytes + i as u64,
                                cursor,
                                expected_byte,
                                received
                            );
                        }

                        cursor += 1;
                        if cursor >= exp.len() {
                            cursor = 0;
                        }
                    }
                }

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

    Ok(TcpStreamStats {
        stream_id,
        bytes_sent: 0,
        bytes_received: bytes,
        duration_ns,
    })
}
