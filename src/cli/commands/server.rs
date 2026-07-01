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
}

impl From<ServerArgs> for ServerCliArgs {
    fn from(args: ServerArgs) -> Self {
        Self {
            local_iface: args.iface,
            local_port: args.port,
            local_addr: args.addr,
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
}

impl ServerArgs {
    pub fn run(self) -> Result<()> {
        super::ensure_root()?;
        server::run(self.into())
    }
}
