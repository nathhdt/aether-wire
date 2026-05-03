use anyhow::Result;
use std::io::{ErrorKind, Read};
use std::net::{IpAddr, SocketAddr, TcpListener};
use std::time::Instant;

use crate::cli::ServeArgs;
use crate::utils::{human_bps, human_bytes};

pub fn run(args: ServeArgs) -> Result<()> {
    let addr = SocketAddr::new(IpAddr::V4(args.bind), args.port);
    let listener = TcpListener::bind(addr)?;
    println!("server listening on {addr}");

    // waits for a client
    let (mut sock, client) = listener.accept()?;
    println!("accepted connection from {client}");

    // read buffer (no optimization, acts like a standard application)
    let mut buf = vec![0u8; 256 * 1024];

    // received bytes
    let mut bytes: u64 = 0;

    // timer & data
    let mut first_byte: Option<Instant> = None;
    let mut last = Instant::now();

    // receive loop
    loop {
        match sock.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => {
                if first_byte.is_none() {
                    first_byte = Some(Instant::now());
                }
                last = Instant::now();
                bytes += n as u64;
            }
            Err(e) if e.kind() == ErrorKind::Interrupted => continue, // EINTR
            Err(e) => return Err(e.into()),
        }
    }

    // elapsed time
    let elapsed = match first_byte {
        Some(t0) => last.duration_since(t0).as_secs_f64(),
        None => 0.0,
    };

    // bitrate
    let bitrate = if elapsed > 0.0 {
        (bytes as f64) * 8.0 / elapsed
    } else {
        0.0
    };

    // result print
    println!("duration : {elapsed:.2}s");
    println!("received : {} ({} bytes)", human_bytes(bytes), bytes);
    println!("bitrate  : {}", human_bps(bitrate));

    Ok(())
}
