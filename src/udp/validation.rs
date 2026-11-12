//! UDP benchmark configuration validation module

use anyhow::{Result, anyhow, bail};
use std::fmt;
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
        interfaces::{get_ready_interface, is_addr_configured, types::Interface},
        resolve::resolve,
    },
};

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct UdpConfig {
    pub server_hostname: Option<String>,
    pub server_addr: IpAddr,
    pub server_port: u16,
    pub client_iface: Interface,
    pub client_source_addr: Option<IpAddr>,

    pub mgmt_server_hostname: Option<String>,
    pub mgmt_server_addr: IpAddr,
    pub mgmt_server_port: u16,
    pub mgmt_client_iface: Interface,
    pub mgmt_client_source_addr: IpAddr,

    pub bandwidth: u64,
    pub length: u16,
    pub duration_secs: u64,
    pub streams: u16,
}

/// distinguishes benchmark server endpoint from management server endpoint in error messages
enum EndpointKind {
    Server,
    ManagementServer,
}

impl fmt::Display for EndpointKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Server => write!(f, "server"),
            Self::ManagementServer => write!(f, "management server"),
        }
    }
}

/// selects candidate address matching IP family of reference
fn pick_compatible_addr(candidates: &[IpAddr], reference: IpAddr) -> Option<IpAddr> {
    candidates
        .iter()
        .copied()
        .find(|addr| addr.is_ipv4() == reference.is_ipv4())
}

/// resolves host using 'source_addr' or interface compatibility
fn resolve_compatible_server_addr(
    host: &str,
    iface: &Interface,
    source_addr: Option<IpAddr>,
    kind: EndpointKind,
) -> Result<(Option<String>, IpAddr)> {
    let hostname = host.parse::<IpAddr>().err().map(|_| host.to_string());
    let candidates = resolve(host)?;

    let addr = if let Some(source_addr) = source_addr {
        pick_compatible_addr(&candidates, source_addr).ok_or_else(|| {
            anyhow!(
                "source address '{}' is not compatible with {} '{}' (address family mismatch)",
                source_addr,
                kind,
                host
            )
        })?
    } else {
        iface
            .addresses
            .iter()
            .find_map(|iface_addr| pick_compatible_addr(&candidates, iface_addr.addr))
            .ok_or_else(|| {
                anyhow!(
                    "interface '{}' has no address compatible with {} '{}' (no matching address family)",
                    iface.name,
                    kind,
                    host
                )
            })?
    };

    Ok((hostname, addr))
}

/// validates UDP benchmark constraints on client interface
fn check_udp_benchmark_constraints(
    client_iface: &Interface,
    args: &UdpCliArgs,
    server_addr: IpAddr,
) -> Result<()> {
    // requested bandwidth must not exceed interface speed
    if let Some(speed) = client_iface.speed
        && args.bandwidth > speed
    {
        bail!(
            "requested bandwidth ({}) exceeds interface '{}' speed ({})",
            human_bps(args.bandwidth),
            client_iface.name,
            human_bps(speed)
        );
    }

    // UDP payload length must not exceed protocol maximum
    let max_payload = match server_addr {
        IpAddr::V4(_) => IPV4_UDP_MAX_PAYLOAD_LENGTH_BYTES,
        IpAddr::V6(_) => IPV6_UDP_MAX_PAYLOAD_LENGTH_BYTES,
    };

    if args.length > max_payload {
        bail!(
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
        bail!(
            "streams ({}) exceeds available queues on '{}' ({})",
            args.streams,
            client_iface.name,
            max
        );
    }

    // warn if streams beyond CPU count: causes context switching
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
                client_iface.name,
                mtu
            );
        }
    }

    Ok(())
}

impl TryFrom<UdpCliArgs> for UdpConfig {
    type Error = anyhow::Error;

    fn try_from(args: UdpCliArgs) -> Result<Self> {
        // management plan options base validation
        if (args.mgmt_server_host.is_some()
            || args.mgmt_server_port.is_some()
            || args.mgmt_client_iface.is_some()
            || args.mgmt_client_source_addr.is_some())
            && (args.mgmt_server_host.is_none()
                || args.mgmt_server_port.is_none()
                || args.mgmt_client_iface.is_none())
        {
            bail!(
                "'--mgmt-server', '--mgmt-port' and '--mgmt-iface' options must be specified together"
            );
        }

        // benchmark interface validation
        let client_iface = get_ready_interface(&args.client_iface)?;

        // if no management server host provided then use benchmark server host
        let mgmt_server_host: String = args
            .mgmt_server_host
            .clone()
            .unwrap_or_else(|| args.server_host.clone());

        // if no management server port provided then use benchmark server port
        let mgmt_server_port = args.mgmt_server_port.unwrap_or(args.server_port);

        // if no management interface provided then use benchmark interface
        let mgmt_client_iface_name = args
            .mgmt_client_iface
            .clone()
            .unwrap_or_else(|| args.client_iface.clone());

        // management link skips UDP benchmark checks
        let mgmt_client_iface = if mgmt_client_iface_name == args.client_iface {
            client_iface.clone()
        } else {
            // management interface validation
            get_ready_interface(&mgmt_client_iface_name)?
        };

        // --mgmt-source-ip address must be configured on the interface
        if let Some(source_addr) = args.mgmt_client_source_addr {
            is_addr_configured(&mgmt_client_iface, source_addr)?;
        }

        // resolves management source address
        let mgmt_client_source_addr = match args.mgmt_client_source_addr {
            Some(addr) => Some(addr),
            None if mgmt_client_iface_name == args.client_iface => args
                .client_source_addr
                .or_else(|| mgmt_client_iface.addresses.first().map(|a| a.addr)),
            None => mgmt_client_iface.addresses.first().map(|a| a.addr),
        }
        .ok_or_else(|| anyhow!("no address available on interface '{mgmt_client_iface_name}'"))?;

        // select management server address compatible with source/interface address
        let (mgmt_server_hostname, mgmt_server_addr) = resolve_compatible_server_addr(
            &mgmt_server_host,
            &mgmt_client_iface,
            Some(mgmt_client_source_addr),
            EndpointKind::ManagementServer,
        )?;

        // --source-ip address must be configured on the interface
        if let Some(source_addr) = args.client_source_addr {
            is_addr_configured(&client_iface, source_addr)?;
        }

        // select server address compatible with source/interface address
        let (server_hostname, server_addr) = resolve_compatible_server_addr(
            &args.server_host,
            &client_iface,
            args.client_source_addr,
            EndpointKind::Server,
        )?;

        // UDP benchmark specific constraints (only applies to benchmark link)
        check_udp_benchmark_constraints(&client_iface, &args, server_addr)?;

        Ok(Self {
            server_hostname,
            server_addr,
            server_port: args.server_port,
            client_iface,
            client_source_addr: args.client_source_addr,

            mgmt_server_hostname,
            mgmt_server_addr,
            mgmt_server_port,
            mgmt_client_iface,
            mgmt_client_source_addr,

            bandwidth: args.bandwidth,
            length: args.length,
            duration_secs: args.duration_secs,
            streams: args.streams,
        })
    }
}
