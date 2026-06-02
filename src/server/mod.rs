//! ather-wire server entrypoint

use anyhow::Result;

use crate::cli::commands::server::ServerConfig;

pub fn run(config: ServerConfig) -> Result<()> {
    println!("server ({}:{})", config.bind_ip, config.port);
    Ok(())
}
