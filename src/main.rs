use anyhow::Result;
use clap::Parser;

use crate::cli::{ClientCommand, Ipv4ClientCommand, Ipv4ServerCommand, ServerCommand};

mod cli;
mod control;
mod tcp;
mod udp;
mod utils;

fn main() -> Result<()> {
    let cli = cli::Cli::parse();

    match cli.command {
        cli::Command::Client(client) => match client {
            ClientCommand::Ipv4(cmd) => match cmd {
                Ipv4ClientCommand::Tcp(args) => tcp::client::run(args),
                Ipv4ClientCommand::Udp(args) => udp::client::run(args),
            },
        },
        cli::Command::Server(server) => match server {
            ServerCommand::Ipv4(cmd) => match cmd {
                Ipv4ServerCommand::Tcp(args) => tcp::server::run(args),
                Ipv4ServerCommand::Udp(args) => udp::server::run(args),
            },
        },
    }
}
