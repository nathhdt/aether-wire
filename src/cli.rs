//! command line interface logic and subcommand definitions

use clap::{Args, Parser, Subcommand};
use std::net::Ipv4Addr;
use std::time::Duration;

/// aether-wire base command
#[derive(Debug, Parser)]
#[command(
    name = env!("CARGO_BIN_NAME"),
    version,
    about = "native cross-platform E2E network performance and benchmarking tool",
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

/// aether-wire commands
#[derive(Debug, Subcommand)]
pub enum Command {
    /// start an aether-wire server
    Server(ServerArgs),

    /// run client operations
    #[command(subcommand)]
    Client(ClientCommand),
}

/// server arguments
#[derive(Args, Clone, Debug)]
pub struct ServerArgs {
    /// IPv4 address to bind to
    #[arg(short = 'b', long, default_value = "0.0.0.0")]
    pub bind: Ipv4Addr,

    /// port to listen on
    #[arg(short = 'p', long)]
    pub port: u16,

    /// exit after serving one session
    #[arg(long)]
    pub once: bool,
}

/// client subcommands
#[derive(Debug, Subcommand)]
pub enum ClientCommand {
    /// run a benchmark (raw TCP throughput measurement)
    Benchmark(BenchmarkArgs),

    /// run full link qualification pipeline
    Qualify(QualifyArgs),
}

/// benchmark mode arguments
#[derive(Args, Clone, Debug)]
pub struct BenchmarkArgs {
    /// server IPv4 address to connect to
    #[arg(short = 's', long, value_parser = validate_server_ipv4)]
    pub server: Ipv4Addr,

    /// server port number
    #[arg(short = 'p', long)]
    pub port: u16,

    /// test duration (minimum 1s)
    #[arg(short = 't', long, default_value = "10s", value_parser = parse_duration_min_1s)]
    pub time: Duration,

    /// number of parallel streams (1-128)
    #[arg(short = 'n', long, default_value_t = 1, value_parser = clap::value_parser!(u16).range(1..=128))]
    pub n_streams: u16,

    /// verify data integrity (reduces throughput, use for diagnostics)
    #[arg(long)]
    pub verify: bool,

    /// traffic direction mode
    #[command(flatten)]
    pub direction: DirectionArgs,
}

/// direction arguments
#[derive(Args, Clone, Debug)]
#[group(required = false, multiple = false)]
pub struct DirectionArgs {
    /// test client <- server throughput
    #[arg(long)]
    pub reverse: bool,

    /// test both directions sequentially
    #[arg(long)]
    pub both: bool,

    /// test both directions simultaneously
    #[arg(long)]
    pub bidirectional: bool,
}

impl DirectionArgs {
    /// converts CLI direction args to protocol Direction enum
    pub fn to_direction(&self) -> crate::protocol::messages::Direction {
        use crate::protocol::messages::Direction;

        if self.reverse {
            Direction::Reverse
        } else if self.both {
            Direction::Both
        } else if self.bidirectional {
            Direction::Bidirectional
        } else {
            Direction::Default
        }
    }
}

/// qualify mode arguments
#[derive(Args, Clone, Debug)]
pub struct QualifyArgs {
    /// server IPv4 address to connect to
    #[arg(short = 's', long, value_parser = validate_server_ipv4)]
    pub server: Ipv4Addr,

    /// server port number
    #[arg(short = 'p', long)]
    pub port: u16,

    /// export full metrics to JSON file
    #[arg(long)]
    pub json: bool,
}

/// checks for a minimal duration of 1s
fn parse_duration_min_1s(s: &str) -> Result<Duration, String> {
    let d = humantime::parse_duration(s).map_err(|e| e.to_string())?;

    if d < Duration::from_secs(1) {
        return Err("duration must be at least 1s".to_string());
    }

    Ok(d)
}

/// validates the provided server IPv4 is an actual reachable host
fn validate_server_ipv4(s: &str) -> Result<Ipv4Addr, String> {
    let ip: Ipv4Addr = s
        .parse()
        .map_err(|_| format!("{s} is not a valid IPv4 address"))?;

    if ip.is_unspecified() {
        return Err("0.0.0.0 is not a valid host address".into());
    }

    if ip.is_multicast() {
        return Err("multicast addresses are not valid hosts".into());
    }

    if ip.octets() == [255, 255, 255, 255] {
        return Err("broadcast addresses is not a valid host".into());
    }

    Ok(ip)
}
