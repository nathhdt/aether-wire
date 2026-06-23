//! aether-wire UDP benchmark entrypoint

mod validator;

use anyhow::Result;

use crate::cli::commands::udp::UdpConfig;
use crate::utils::format::human_bps;

pub fn run(config: UdpConfig) -> Result<()> {
    validator::validate_config(&config)?;

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
