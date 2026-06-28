//! aether-wire UDP benchmark entrypoint

mod validator;

use anyhow::Result;

use crate::cli::commands::udp::UdpConfig;
use crate::err_info;
use crate::utils::format::human_bps;

pub fn run(config: UdpConfig) -> Result<()> {
    validator::validate_config(&config)?;

    let source_ip = config
        .source_addr
        .map(|ip| ip.to_string())
        .unwrap_or_else(|| "auto".to_string());

    err_info!("UDP performance test");
    err_info!("    server:            {}:{}", config.server, config.port);
    err_info!("    interface:         {}", config.iface);
    err_info!("    source IP:         {}", source_ip);
    err_info!("    bandwidth:         {}", human_bps(config.bandwidth));
    err_info!("    payload size:      {} bytes", config.length);
    err_info!("    duration:          {}s", config.duration_secs);
    err_info!("    parallel streams:  {}", config.streams);

    Ok(())
}
