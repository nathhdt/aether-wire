//! aether-wire server entrypoint

mod validator;

use anyhow::Result;

use crate::cli::commands::server::ServerConfig;
use crate::utils::logging::info;

pub fn run(config: ServerConfig) -> Result<()> {
    validator::validate_config(&config)?;

    let listen_addr = config
        .local_addr
        .map(|ip| ip.to_string())
        .unwrap_or_else(|| "all addresses".to_string());

    info!("benchmarking server");
    info!("    listen:            {}:{}", listen_addr, config.port);
    info!("    interface:         {}", config.iface);

    Ok(())
}
