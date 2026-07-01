//! aether-wire server entrypoint

mod config;

use anyhow::Result;

use crate::cli::commands::server::ServerCliArgs;
use crate::log_info;

pub fn run(args: ServerCliArgs) -> Result<()> {
    let _config = config::ServerConfig::try_from(args)?;

    log_info!("benchmarking server");

    Ok(())
}
