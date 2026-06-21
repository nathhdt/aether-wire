//! server subcommand arguments and execution

use anyhow::Result;
use clap::{Args, value_parser};
use std::net::IpAddr;

use crate::server;

#[derive(Debug)]
pub struct ServerConfig {
    pub iface: String,
    pub port: u16,
    pub source_addr: Option<IpAddr>,
}

impl From<ServerArgs> for ServerConfig {
    fn from(args: ServerArgs) -> Self {
        Self {
            iface: args.iface,
            port: args.port,
            source_addr: args.source_addr,
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
        short = 's',
        long = "source",
        value_name = "ip",
        help = "source IP address"
    )]
    source_addr: Option<IpAddr>,
}

impl ServerArgs {
    pub fn run(self) -> Result<()> {
        super::ensure_root()?;
        server::run(self.into())
    }
}
