use anyhow::Result;
use clap::Parser;

use crate::cli::{ClientCommand, Ipv4ClientCommand, Ipv4ServerCommand, ServerCommand};

mod cli;
mod ipv4_tcp_client;
mod ipv4_tcp_server;
mod ipv4_udp_client;
mod ipv4_udp_server;
mod payload;
mod proto;
mod tcp_utils;
mod utils;
mod wire;

fn main() -> Result<()> {
    let cli = cli::Cli::parse();

    match cli.command {
        cli::Command::Client(client) => match client {
            ClientCommand::Ipv4(cmd) => match cmd {
                Ipv4ClientCommand::Tcp(args) => ipv4_tcp_client::run(args),
                Ipv4ClientCommand::Udp(args) => ipv4_udp_client::run(args),
            },
        },
        cli::Command::Server(server) => match server {
            ServerCommand::Ipv4(cmd) => match cmd {
                Ipv4ServerCommand::Tcp(args) => ipv4_tcp_server::run(args),
                Ipv4ServerCommand::Udp(args) => ipv4_udp_server::run(args),
            },
        },
    }
}
