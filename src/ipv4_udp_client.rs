//! aether-wire UDP client

use anyhow::Result;

use crate::cli::Ipv4UdpClientArgs;

/// runs the UDP client, connects to a server, and benchmarks the wire
pub fn run(args: Ipv4UdpClientArgs) -> Result<()> {
    println!(
        "UDP client to server {}:{} (not implemented yet)",
        args.server, args.port
    );

    Ok(())
}
