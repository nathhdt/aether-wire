//! udp subcommand arguments and execution

use anyhow::Result;
use clap::{Args, value_parser};
use std::net::IpAddr;

use crate::cli::parsing::parse_bandwidth;
use crate::udp;

#[derive(Debug)]
pub struct UdpConfig {
    pub server_ip: IpAddr,
    pub port: u16,
    pub length: u16,
    pub bandwidth: u64,
}

impl From<UdpArgs> for UdpConfig {
    fn from(args: UdpArgs) -> Self {
        Self {
            server_ip: args.server_ip,
            port: args.port,
            length: args.length,
            bandwidth: args.bandwidth,
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
pub struct UdpArgs {
    #[arg(
        short = 's',
        long = "server",
        value_name = "ip",
        help = "target server IP address"
    )]
    server_ip: IpAddr,

    #[arg(
        short = 'p',
        long = "port",
        value_name = "port",
        value_parser = value_parser!(u16).range(1..),
        help = "target server port"
    )]
    port: u16,

    #[arg(
        short = 'l',
        long = "length",
        value_name = "bytes",
        value_parser = value_parser!(u16).range(1..=65507),
        help = "payload length"
    )]
    length: u16,

    #[arg(
        short = 'b',
        long = "bandwidth",
        value_name = "rate",
        value_parser = parse_bandwidth,
        help = "bandwidth limit (e.g. 400K, 20M, 1G)"
    )]
    bandwidth: u64,
}

impl UdpArgs {
    pub fn run(self) -> Result<()> {
        udp::run(self.into())
    }
}
