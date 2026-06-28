//! aether-wire UDP benchmark entrypoint

mod validator;

use anyhow::Result;

use crate::cli::commands::udp::UdpConfig;
use crate::log_info;
use crate::utils::format::human_bps;

pub fn run(config: UdpConfig) -> Result<()> {
    validator::validate_config(&config)?;

    let source_ip = config
        .source_addr
        .map(|ip| ip.to_string())
        .unwrap_or_else(|| "auto".to_string());

    log_info!("UDP performance test");
    log_info!("    server:            {}:{}", config.server, config.port);
    log_info!("    interface:         {}", config.iface);
    log_info!("    source IP:         {}", source_ip);
    log_info!("    bandwidth:         {}", human_bps(config.bandwidth));
    log_info!("    payload size:      {} bytes", config.length);
    log_info!("    duration:          {}s", config.duration_secs);
    log_info!("    parallel streams:  {}", config.streams);

    Ok(())
}
