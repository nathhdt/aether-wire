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

    #[arg(short = 't', long, default_value = "10s", value_parser = parse_duration_min_1s)]
    pub time: Duration,

    #[arg(short = 'n', long, default_value_t = 1, value_parser = clap::value_parser!(u16).range(1..=128))]
    pub n_streams: u16,
}

fn parse_duration_min_1s(s: &str) -> Result<std::time::Duration, String> {
    let d = humantime::parse_duration(s).map_err(|e| e.to_string())?;

    if d < std::time::Duration::from_secs(1) {
        return Err("duration must be at least 1s".to_string());
    }

    Ok(d)
}
