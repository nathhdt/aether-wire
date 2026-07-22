//! aether-wire UDP benchmark entrypoint

mod validation;

use anyhow::Result;

use crate::cli::commands::udp::UdpCliArgs;
use crate::log_info;

pub fn run(args: UdpCliArgs) -> Result<()> {
    let _config = validation::UdpConfig::try_from(args)?;

    log_info!("UDP performance test");

    Ok(())
}
