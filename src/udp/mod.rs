//! aether-wire UDP benchmark entrypoint

use anyhow::Result;

use crate::cli::commands::udp::UdpConfig;

pub fn run(config: UdpConfig) -> Result<()> {
    println!(
        "UDP client ({}:{}), payload size: {}, requested bandwidth: {}",
        config.server_ip, config.port, config.length, config.bandwidth
    );

    Ok(())
}
