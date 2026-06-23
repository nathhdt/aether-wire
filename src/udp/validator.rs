//! UDP benchmark config validator

use anyhow::Result;

use crate::cli::commands::udp::UdpConfig;
use crate::utils::network::constants::IF_OPER_UP;
use crate::utils::network::interfaces::{
    get_interface, get_interface_addresses, get_interface_details,
};

pub fn validate_config(config: &UdpConfig) -> Result<()> {
    // check if interface exists
    let iface = get_interface(&config.iface)?;

    let details = get_interface_details(iface.index)?
        .ok_or_else(|| anyhow::anyhow!("no Netlink data for interface '{}'", config.iface))?;

    // interface must be up
    if details.operstate != Some(IF_OPER_UP) {
        anyhow::bail!("interface '{}' is not up", config.iface);
    }

    // interface must have at least one address
    let addresses = get_interface_addresses(iface.index)?;
    if addresses.is_empty() {
        anyhow::bail!("interface '{}' has no configured addresses", config.iface);
    }

    Ok(())
}
