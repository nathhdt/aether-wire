//! aether-wire UDP benchmark entrypoint

use anyhow::Result;

use crate::cli::commands::udp::UdpConfig;
use crate::utils::format::human_bps;
use crate::utils::network::constants::IF_OPER_UP;
use crate::utils::network::interfaces::{
    get_interface, get_interface_addresses, get_interface_details,
};

pub fn run(config: UdpConfig) -> Result<()> {
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

    println!(
        "UDP client ({}:{}), interface: {}, requested bandwidth: {}, payload size: {}, duration: {}s, {} streams",
        config.server,
        config.port,
        config.iface,
        human_bps(config.bandwidth),
        config.length,
        config.duration_secs,
        config.streams,
    );

    Ok(())
}
