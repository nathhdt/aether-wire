//! server configuration validation module

use anyhow::{Result, bail};
use std::net::IpAddr;

use crate::cli::commands::server::ServerCliArgs;
use crate::utils::network::interfaces::{
    get_benchmark_ready_interface, is_addr_configured, types::Interface,
};

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub local_iface: Interface,
    pub local_port: u16,
    pub local_addr: Option<IpAddr>,

    pub mgmt_iface: Interface,
    pub mgmt_port: u16,
    pub mgmt_addr: Option<IpAddr>,
}

impl TryFrom<ServerCliArgs> for ServerConfig {
    type Error = anyhow::Error;

    fn try_from(args: ServerCliArgs) -> Result<Self> {
        // management plan base validation
        if args.mgmt_iface.is_some() && args.mgmt_port.is_none() && args.mgmt_addr.is_none() {
            bail!("'--mgmt-iface' requires at least one of '--mgmt-port' or '--mgmt-ip' options");
        }

        // check base interface requirements for benchmark
        let local_iface = get_benchmark_ready_interface(&args.local_iface)?;

        // --local-ip address must be configured on the interface
        if let Some(local_addr) = args.local_addr {
            is_addr_configured(&local_iface, local_addr)?;
        }

        // reuses already-validated iface if same for management
        let mgmt_iface_name = args
            .mgmt_iface
            .clone()
            .unwrap_or_else(|| args.local_iface.clone());

        let mgmt_iface = if mgmt_iface_name == args.local_iface {
            local_iface.clone()
        } else {
            // check base interface requirements for management
            get_benchmark_ready_interface(&mgmt_iface_name)?
        };

        // --mgmt-ip address must be configured on the interface
        if let Some(mgmt_addr) = args.mgmt_addr {
            is_addr_configured(&mgmt_iface, mgmt_addr)?;
        }

        let mgmt_port = args.mgmt_port.unwrap_or(args.local_port);

        Ok(Self {
            local_iface,
            local_port: args.local_port,
            local_addr: args.local_addr,

            mgmt_iface,
            mgmt_port,
            mgmt_addr: args.mgmt_addr,
        })
    }
}
