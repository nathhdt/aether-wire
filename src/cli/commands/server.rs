//! server subcommand arguments and execution

use anyhow::Result;
use clap::{Args, value_parser};
use std::net::IpAddr;

use crate::server;

#[derive(Debug)]
pub struct ServerCliArgs {
    pub local_iface: String,
    pub local_port: u16,
    pub local_addr: Option<IpAddr>,

    pub mgmt_iface: Option<String>,
    pub mgmt_port: Option<u16>,
    pub mgmt_addr: Option<IpAddr>,
}

impl From<ServerArgs> for ServerCliArgs {
    fn from(args: ServerArgs) -> Self {
        Self {
            local_iface: args.iface,
            local_port: args.port,
            local_addr: args.addr,

            mgmt_iface: args.mgmt_iface,
            mgmt_port: args.mgmt_port,
            mgmt_addr: args.mgmt_addr,
        }
    }
}

#[derive(Args, Debug)]
#[command(
    arg_required_else_help = true,
    help_template = "\
{about-with-newline}
usage: {usage}

options:
{options}
"
)]
pub struct ServerArgs {
    #[arg(
        short = 'i',
        long = "iface",
        value_name = "interface",
        help = "network interface to use"
    )]
    iface: String,

    #[arg(
        short = 'p',
        long = "port",
        value_name = "port",
        value_parser = value_parser!(u16).range(1..),
        help = "port to listen on"
    )]
    port: u16,

    #[arg(
        short = 'l',
        long = "local-ip",
        value_name = "ip",
        help = "local IP address to use [default: first address on selected interface]"
    )]
    addr: Option<IpAddr>,

    #[arg(
        long = "mgmt-iface",
        value_name = "interface",
        help = "network interface to use for management traffic [default: same as --iface]"
    )]
    mgmt_iface: Option<String>,

    #[arg(
        long = "mgmt-port",
        value_name = "port",
        value_parser = value_parser!(u16).range(1..),
        requires = "mgmt_iface",
        help = "management port to listen on [default: same as --port]"
    )]
    mgmt_port: Option<u16>,

    #[arg(
        long = "mgmt-ip",
        value_name = "ip",
        requires = "mgmt_iface",
        help = "local management IP address to use [default: first address on selected management interface]"
    )]
    mgmt_addr: Option<IpAddr>,
}

impl ServerArgs {
    pub fn run(self) -> Result<()> {
        super::ensure_root()?;
        server::run(self.into())
    }
}
