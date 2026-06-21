//! aether-wire UDP benchmark entrypoint

use anyhow::Result;

use crate::cli::commands::udp::UdpConfig;
use crate::utils::format::human_bps;

pub fn run(config: UdpConfig) -> Result<()> {
    println!(
        "UDP client ({}:{}), payload size: {}, requested bandwidth: {}, duration: {}, {} streams, interface: {}",
        config.server,
        config.port,
        config.length,
        human_bps(config.bandwidth),
        config.duration_secs,
        config.streams,
        config.iface.as_deref().unwrap_or("auto"),
    );

    Ok(())
}
