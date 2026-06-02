//! server subcommand arguments and execution

use anyhow::Result;
use clap::{Args, value_parser};
use std::net::IpAddr;

use crate::server;

#[derive(Debug)]
pub struct ServerConfig {
    pub bind_ip: IpAddr,
    pub port: u16,
}

impl From<ServerArgs> for ServerConfig {
    fn from(args: ServerArgs) -> Self {
        Self {
            bind_ip: args.bind_ip,
            port: args.port,
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
        short = 'b',
        long = "bind",
        value_name = "ip",
        help = "IP address to bind to"
    )]
    bind_ip: IpAddr,

    #[arg(
        short = 'p',
        long = "port",
        value_name = "port",
        value_parser = value_parser!(u16).range(1..),
        help = "port to listen on"
    )]
    port: u16,
}

impl ServerArgs {
    pub fn run(self) -> Result<()> {
        server::run(self.into())
    }
}
