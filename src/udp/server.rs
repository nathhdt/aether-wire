//! aether-wire UDP server

use anyhow::Result;

use crate::cli::Ipv4UdpServerArgs;

/// runs the UDP server, listens for a connection, and benchmarks the wire
pub fn run(args: Ipv4UdpServerArgs) -> Result<()> {
    println!(
        "UDP server {}:{} (not implemented yet)",
        args.bind, args.port
    );

    Ok(())
}
