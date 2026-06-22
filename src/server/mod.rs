//! aether-wire server entrypoint

use anyhow::Result;

use crate::cli::commands::server::ServerConfig;
use crate::utils::network::interfaces::interface_exists;

pub fn run(config: ServerConfig) -> Result<()> {
    // check if interface exists
    if !config.iface.is_empty() && !interface_exists(&config.iface) {
        anyhow::bail!("interface '{}' not found", config.iface);
    }

    println!(
        "server ({:?}:{}), interface: {}",
        config.source_addr, config.port, config.iface
    );

    Ok(())
}
