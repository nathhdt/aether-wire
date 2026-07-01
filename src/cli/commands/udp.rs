//! udp subcommand arguments and execution

use anyhow::Result;
use clap::{Args, value_parser};
use std::net::IpAddr;

use crate::udp;

use super::super::parsing::{parse_bandwidth, parse_duration, parse_udp_payload_length};

#[derive(Debug)]
pub struct UdpCliArgs {
    pub server_host: String,
    pub server_port: u16,
    pub client_iface: String,
    pub client_source_addr: Option<IpAddr>,
    pub bandwidth: u64,
    pub length: u16,
    pub duration_secs: u64,
    pub streams: u16,
}

impl From<UdpArgs> for UdpCliArgs {
    fn from(args: UdpArgs) -> Self {
        Self {
            server_host: args.server,
            server_port: args.port,
            client_iface: args.iface,
            client_source_addr: args.source_addr,
            bandwidth: args.bandwidth,
            length: args.length,
            duration_secs: args.duration_secs,
            streams: args.streams,
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
        value_name = "host",
        help = "target server hostname or IP address"
    )]
    server: String,

    #[arg(
        short = 'p',
        long = "port",
        value_name = "port",
        value_parser = value_parser!(u16).range(1..),
        help = "target server port"
    )]
    port: u16,

    #[arg(
        short = 'i',
        long = "iface",
        value_name = "interface",
        help = "network interface to use"
    )]
    iface: String,

    #[arg(
        short = 'S',
        long = "source-ip",
        value_name = "ip",
        help = "source IP address to use [default: first address on selected interface]"
    )]
    source_addr: Option<IpAddr>,

    #[arg(
        short = 'b',
        long = "bandwidth",
        value_name = "rate",
        value_parser = parse_bandwidth,
        help = "bandwidth limit (e.g. 400K, 20M, 1G)"
    )]
    bandwidth: u64,

    #[arg(
        short = 'l',
        long = "length",
        value_name = "bytes",
        value_parser = parse_udp_payload_length,
        help = "UDP payload length"
    )]
    length: u16,

    #[arg(
        short = 't',
        long = "duration",
        value_name = "duration",
        value_parser = parse_duration,
        default_value = "10s",
        help = "test duration (e.g. 8s, 4m, 2h, 1d)"
    )]
    duration_secs: u64,

    #[arg(
        short = 'n',
        long = "streams",
        value_name = "count",
        value_parser = value_parser!(u16).range(1..),
        default_value_t = 1,
        help = "number of parallel UDP streams"
    )]
    streams: u16,
}

impl UdpArgs {
    pub fn run(self) -> Result<()> {
        super::ensure_root()?;
        udp::run(self.into())
    }
}
