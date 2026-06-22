//! aether-wire UDP benchmark entrypoint

use anyhow::Result;

use crate::cli::commands::udp::UdpConfig;
use crate::utils::format::human_bps;
use crate::utils::network::interfaces::interface_exists;

pub fn run(config: UdpConfig) -> Result<()> {
    // check if interface exists
    if !config.iface.is_empty() && !interface_exists(&config.iface) {
        anyhow::bail!("interface '{}' not found", config.iface);
    }

    println!(
        "UDP client ({}:{}), interface: {}, requested bandwidth: {}, payload size: {}, duration: {}s, {} streams",
        config.server,
        config.port,
        config.iface,
        human_bps(config.bandwidth),
        config.length,
        config.duration_secs,
        config.streams,
    );

    Ok(())
}
