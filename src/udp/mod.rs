//! aether-wire UDP benchmark entrypoint

mod validator;

use anyhow::Result;

use crate::cli::commands::udp::UdpConfig;
use crate::utils::format::human_bps;
use crate::utils::logging::info;

pub fn run(config: UdpConfig) -> Result<()> {
    validator::validate_config(&config)?;

    let source_ip = config
        .source_addr
        .map(|ip| ip.to_string())
        .unwrap_or_else(|| "auto".to_string());

    info!("UDP performance test");
    info!("    server:            {}:{}", config.server, config.port);
    info!("    interface:         {}", config.iface);
    info!("    source IP:         {}", source_ip);
    info!("    bandwidth:         {}", human_bps(config.bandwidth));
    info!("    payload size:      {} bytes", config.length);
    info!("    duration:          {}s", config.duration_secs);
    info!("    parallel streams:  {}", config.streams);

    Ok(())
}
