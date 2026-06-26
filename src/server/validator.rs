//! server configuration validator

use anyhow::Result;

use crate::cli::commands::server::ServerConfig;
use crate::utils::network::interfaces::constants::IF_OPER_UP;
use crate::utils::network::interfaces::get_interface;

pub fn validate_config(config: &ServerConfig) -> Result<()> {
    let iface = get_interface(&config.iface)?;

    // interface must be up
    if iface.operstate != Some(IF_OPER_UP) {
        anyhow::bail!("interface '{}' is not up", config.iface);
    }

    // interface must have at least one address
    if iface.addresses.is_empty() {
        anyhow::bail!("interface '{}' has no configured addresses", config.iface);
    }

    // --local-ip address must be configured on the interface
    if let Some(local_addr) = config.local_addr
        && !iface.addresses.iter().any(|a| a.addr == local_addr)
    {
        anyhow::bail!(
            "address '{}' is not configured on interface '{}'",
            local_addr,
            config.iface
        );
    }

    Ok(())
}
