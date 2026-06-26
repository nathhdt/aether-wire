//! UDP benchmark configuration validator

use anyhow::Result;
use std::net::IpAddr;

use crate::cli::commands::udp::UdpConfig;
use crate::protocol::constants::{
    ETHERNET_IPV4_UDP_OVERHEAD_BYTES, ETHERNET_IPV6_UDP_OVERHEAD_BYTES,
};
use crate::utils::format::human_bps;
use crate::utils::logging::warn;
use crate::utils::network::interfaces::constants::IF_OPER_UP;
use crate::utils::network::interfaces::get_interface;
use crate::utils::network::resolve::resolve;

pub fn validate_config(config: &UdpConfig) -> Result<()> {
    let iface = get_interface(&config.iface)?;

    // interface must be up
    if iface.operstate != Some(IF_OPER_UP) {
        anyhow::bail!("interface '{}' is not up", config.iface);
    }

    // requested bandwidth must not exceed interface speed
    if let Some(speed) = iface.speed
        && config.bandwidth > speed
    {
        anyhow::bail!(
            "requested bandwidth ({}) exceeds interface '{}' speed ({})",
            human_bps(config.bandwidth),
            config.iface,
            human_bps(speed)
        );
    }

    // interface must have at least one address
    if iface.addresses.is_empty() {
        anyhow::bail!("interface '{}' has no configured addresses", config.iface);
    }

    // --source-ip address must be configured on the interface
    if let Some(source_addr) = config.source_addr
        && !iface.addresses.iter().any(|a| a.addr == source_addr)
    {
        anyhow::bail!(
            "address '{}' is not configured on interface '{}'",
            source_addr,
            config.iface
        );
    }

    // source address family must match server address family
    let server_addr = resolve(&config.server)?;

    if let Some(source_addr) = config.source_addr {
        let compatible = matches!(
            (source_addr, server_addr),
            (IpAddr::V4(_), IpAddr::V4(_)) | (IpAddr::V6(_), IpAddr::V6(_))
        );
        if !compatible {
            anyhow::bail!(
                "source address '{}' is not compatible with server '{}' (address family mismatch)",
                source_addr,
                config.server
            );
        }
    } else {
        let compatible_addr = iface.addresses.iter().find(|a| {
            matches!(
                (a.addr, server_addr),
                (IpAddr::V4(_), IpAddr::V4(_)) | (IpAddr::V6(_), IpAddr::V6(_))
            )
        });
        if compatible_addr.is_none() {
            anyhow::bail!(
                "interface '{}' has no address compatible with server '{}' (no matching address family)",
                config.iface,
                config.server
            );
        }
    }

    // streams must not exceed available queues
    let max_queue_streams = iface
        .rx_queues
        .zip(iface.tx_queues)
        .map(|(rx, tx)| rx.min(tx));

    if let Some(max) = max_queue_streams
        && config.streams as u32 > max
    {
        anyhow::bail!(
            "streams ({}) exceeds available queues on '{}' ({})",
            config.streams,
            config.iface,
            max
        );
    }

    // streams beyond cpu count causes context switching
    let cpu_count = std::thread::available_parallelism()
        .map(|n| n.get() as u32)
        .unwrap_or(u32::MAX);

    if config.streams as u32 > cpu_count {
        warn!(
            "streams ({}) exceeds CPU count ({}), expect context switching",
            config.streams, cpu_count
        );
    }

    // warn if resulting frame exceeds interface MTU
    if let Some(mtu) = iface.mtu {
        let overhead = match resolve(&config.server)? {
            IpAddr::V4(_) => ETHERNET_IPV4_UDP_OVERHEAD_BYTES,
            IpAddr::V6(_) => ETHERNET_IPV6_UDP_OVERHEAD_BYTES,
        };
        let frame_size = config.length as u32 + overhead;

        if frame_size > mtu {
            warn!(
                "frame size ({} bytes) exceeds MTU on '{}' ({} bytes), packets will be fragmented",
                frame_size, config.iface, mtu
            );
        }
    }

    Ok(())
}
