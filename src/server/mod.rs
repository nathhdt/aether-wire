//! aether-wire server entrypoint

mod validator;

use anyhow::Result;

use crate::cli::commands::server::ServerConfig;

pub fn run(config: ServerConfig) -> Result<()> {
    validator::validate_config(&config)?;

    println!(
        "server ({:?}:{}), interface: {}",
        config.source_addr, config.port, config.iface
    );

    Ok(())
}
