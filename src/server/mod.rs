//! aether-wire server entrypoint

use anyhow::Result;

use crate::cli::commands::server::ServerConfig;

pub fn run(config: ServerConfig) -> Result<()> {
    println!(
        "server ({:?}:{}), interface: {}",
        config.source_addr, config.port, config.iface
    );

    Ok(())
}
