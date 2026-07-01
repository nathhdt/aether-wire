//! UDP benchmark configuration module

use anyhow::Result;
use std::net::IpAddr;

use crate::cli::commands::udp::UdpCliArgs;
use crate::log_warn;
use crate::protocol::constants::{
    ETHERNET_IPV4_UDP_OVERHEAD_BYTES, ETHERNET_IPV6_UDP_OVERHEAD_BYTES,
    IPV4_UDP_MAX_PAYLOAD_LENGTH_BYTES, IPV6_UDP_MAX_PAYLOAD_LENGTH_BYTES,
};
use crate::utils::{
    format::human_bps,
    network::{
        interfaces::constants::IF_OPER_UP,
        interfaces::get_interface,
        interfaces::types::Interface,
        resolve::resolve,
    }
};

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct UdpConfig {
    pub server_hostname: Option<String>,
    pub server_addr: IpAddr,
    pub server_port: u16,
    pub client_iface: Interface,
    pub client_source_addr: Option<IpAddr>,
    pub bandwidth: u64,
    pub length: u16,
    pub duration_secs: u64,
    pub streams: u16,
}

impl TryFrom<UdpCliArgs> for UdpConfig {
    type Error = anyhow::Error;

    fn try_from(args: UdpCliArgs) -> Result<Self> {
        let client_iface = get_interface(&args.client_iface)?;

        // interface must be up
        if client_iface.operstate != Some(IF_OPER_UP) {
            anyhow::bail!("interface '{}' is not up", args.client_iface);
        }

        // interface must have at least one address
        if client_iface.addresses.is_empty() {
            anyhow::bail!(
                "interface '{}' has no configured addresses",
                args.client_iface
            );
        }

        // --source-ip address must be configured on the interface
        if let Some(source_addr) = args.client_source_addr
            && !client_iface.addresses.iter().any(|a| a.addr == source_addr)
        {
            anyhow::bail!(
                "address '{}' is not configured on interface '{}'",
                source_addr,
                args.client_iface
            );
        }

        // select server address compatible with source/interface address
        let server_hostname = args
            .server_host
            .parse::<IpAddr>()
            .err()
            .map(|_| args.server_host.clone());

        let server_addrs = resolve(&args.server_host)?;

        let server_addr = if let Some(source_addr) = args.client_source_addr {
            server_addrs
                .iter()
                .copied()
                .find(|server_addr| source_addr.is_ipv4() == server_addr.is_ipv4())
                .ok_or_else(|| {
                    anyhow::anyhow!(
                        "source address '{}' is not compatible with server '{}' (address family mismatch)",
                        source_addr,
                        args.server_host
                    )
                })?
        } else {
            client_iface
                .addresses
                .iter()
                .find_map(|iface_addr| {
                    server_addrs
                        .iter()
                        .copied()
                        .find(|server_addr| iface_addr.addr.is_ipv4() == server_addr.is_ipv4())
                })
                .ok_or_else(|| {
                    anyhow::anyhow!(
                        "interface '{}' has no address compatible with server '{}' (no matching address family)",
                        args.client_iface,
                        args.server_host
                    )
                })?
        };

        // requested bandwidth must not exceed interface speed
        if let Some(speed) = client_iface.speed
            && args.bandwidth > speed
        {
            anyhow::bail!(
                "requested bandwidth ({}) exceeds interface '{}' speed ({})",
                human_bps(args.bandwidth),
                args.client_iface,
                human_bps(speed)
            );
        }

        // UDP payload length must not exceed the protocol maximum
        let max_payload = match server_addr {
            IpAddr::V4(_) => IPV4_UDP_MAX_PAYLOAD_LENGTH_BYTES,
            IpAddr::V6(_) => IPV6_UDP_MAX_PAYLOAD_LENGTH_BYTES,
        };

        if args.length > max_payload {
            anyhow::bail!(
                "UDP payload length ({} bytes) exceeds the maximum for {} ({} bytes)",
                args.length,
                match server_addr {
                    IpAddr::V4(_) => "IPv4",
                    IpAddr::V6(_) => "IPv6",
                },
                max_payload
            );
        }

        // streams must not exceed available queues
        let max_queue_streams = client_iface
            .rx_queues
            .zip(client_iface.tx_queues)
            .map(|(rx, tx)| rx.min(tx));

        if let Some(max) = max_queue_streams
            && args.streams as u32 > max
        {
            anyhow::bail!(
                "streams ({}) exceeds available queues on '{}' ({})",
                args.streams,
                args.client_iface,
                max
            );
        }

        // streams beyond CPU count causes context switching
        let cpu_count = std::thread::available_parallelism()
            .map(|n| n.get() as u32)
            .unwrap_or(u32::MAX);

        if args.streams as u32 > cpu_count {
            log_warn!(
                "streams ({}) exceeds CPU count ({}), expect context switching",
                args.streams,
                cpu_count
            );
        }

        // warn if resulting frame exceeds interface MTU
        if let Some(mtu) = client_iface.mtu {
            let overhead = match server_addr {
                IpAddr::V4(_) => ETHERNET_IPV4_UDP_OVERHEAD_BYTES,
                IpAddr::V6(_) => ETHERNET_IPV6_UDP_OVERHEAD_BYTES,
            };

            let frame_size = args.length as u32 + overhead;

            if frame_size > mtu {
                log_warn!(
                    "frame size ({} bytes) exceeds MTU on '{}' ({} bytes), packets will be fragmented",
                    frame_size,
                    args.client_iface,
                    mtu
                );
            }
        }

        Ok(Self {
            server_hostname,
            server_addr,
            server_port: args.server_port,
            client_iface,
            client_source_addr: args.client_source_addr,
            bandwidth: args.bandwidth,
            length: args.length,
            duration_secs: args.duration_secs,
            streams: args.streams,
        })
    }
}
