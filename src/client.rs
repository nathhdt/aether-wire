use anyhow::Result;
use std::io::{ErrorKind, Write};
use std::net::{Shutdown, IpAddr, SocketAddr, TcpStream};
use std::time::Instant;

use crate::cli::ClientArgs;
use crate::utils::{human_bps, human_bytes};

pub fn run(args: ClientArgs) -> Result<()> {
    let addr = SocketAddr::new(IpAddr::V4(args.server), args.port);
    let mut sock = TcpStream::connect(addr)?;
    println!("connected to {addr}, sending for {:?}", args.time);

    // basic filler payload
    let buf = vec![0xA5u8; 64 * 1024];

    // timer
    let start = Instant::now();
    let deadline = start + args.time;
    let mut bytes: u64 = 0;

    // send loop
    while Instant::now() < deadline {
        match sock.write(&buf) {
            Ok(0) => break,
            Ok(n) => bytes += n as u64,
            Err(e) if e.kind() == ErrorKind::Interrupted => continue, // EINTR
            Err(e) => return Err(e.into()),
        }
    }

    let elapsed = start.elapsed();

    // ends TCP session (sends FIN)
    sock.shutdown(Shutdown::Write)?;

    // bitrate
    let secs = elapsed.as_secs_f64();
    let bitrate = if secs > 0.0 {
        (bytes as f64) * 8.0 / secs
    } else {
        0.0
    };

    // result print
    println!("duration : {secs:.2}s");
    println!("sent     : {} ({} bytes)", human_bytes(bytes), bytes);
    println!("bitrate  : {}", human_bps(bitrate));

    Ok(())
}
