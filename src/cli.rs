//! command line interface logic & subcommand definitions

use clap::{Args, Parser, Subcommand};
use std::net::Ipv4Addr;
use std::time::Duration;

#[derive(Parser, Debug)]
#[command(
    name = env!("CARGO_BIN_NAME"),
    version,
    about,
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    Serve(ServeArgs),
    Client(ClientArgs),
}

#[derive(Args, Clone, Debug)]
pub struct ServeArgs {
    #[arg(short = 'b', long, default_value = "0.0.0.0")]
    pub bind: Ipv4Addr,

    #[arg(short = 'p', long)]
    pub port: u16,
}

#[derive(Args, Clone, Debug)]
pub struct ClientArgs {
    #[arg(short = 's', long)]
    pub server: Ipv4Addr,

    #[arg(short = 'p', long)]
    pub port: u16,

    #[arg(short = 't', long, default_value = "10s", value_parser = humantime::parse_duration)]
    pub time: Duration,

    #[arg(short = 'n', long, default_value_t = 1)]
    pub n_streams: u16,
}
