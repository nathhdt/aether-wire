//! server configuration module

use anyhow::Result;
use std::net::IpAddr;

use crate::cli::commands::server::ServerCliArgs;
use crate::utils::network::interfaces::constants::IF_OPER_UP;
use crate::utils::network::interfaces::get_interface;
use crate::utils::network::interfaces::types::Interface;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub local_iface: Interface,
    pub local_port: u16,
    pub local_addr: Option<IpAddr>,
}

impl TryFrom<ServerCliArgs> for ServerConfig {
    type Error = anyhow::Error;

    fn try_from(args: ServerCliArgs) -> Result<Self> {
        let local_iface = get_interface(&args.local_iface)?;

        // interface must be up
        if local_iface.operstate != Some(IF_OPER_UP) {
            anyhow::bail!("interface '{}' is not up", args.local_iface);
        }

        // interface must have at least one address
        if local_iface.addresses.is_empty() {
            anyhow::bail!(
                "interface '{}' has no configured addresses",
                args.local_iface
            );
        }

        // --local-ip address must be configured on the interface
        if let Some(local_addr) = args.local_addr
            && !local_iface.addresses.iter().any(|a| a.addr == local_addr)
        {
            anyhow::bail!(
                "address '{}' is not configured on interface '{}'",
                local_addr,
                args.local_iface
            );
        }

        Ok(Self {
            local_iface,
            local_port: args.local_port,
            local_addr: args.local_addr,
        })
    }
}
