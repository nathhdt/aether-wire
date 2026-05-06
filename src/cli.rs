//! command line interface logic & subcommand definitions

use clap::{Args, Parser, Subcommand};
use std::net::Ipv4Addr;
use std::time::Duration;

/// aether-wire base command
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

/// aether-wire commands
#[derive(Subcommand, Debug)]
pub enum Command {
    #[command(subcommand)]
    Server(ServerCommand),

    #[command(subcommand)]
    Client(ClientCommand),
}

/// aether-wire server command
#[derive(Subcommand, Debug)]
pub enum ServerCommand {
    #[command(subcommand)]
    Ipv4(Ipv4ServerCommand),
}

/// aether-wire client command
#[derive(Subcommand, Debug)]
pub enum ClientCommand {
    #[command(subcommand)]
    Ipv4(Ipv4ClientCommand),
}

/// aether-wire ipv4 server command
#[derive(Subcommand, Debug)]
pub enum Ipv4ServerCommand {
    Tcp(Ipv4TcpServerArgs),
    Udp(Ipv4UdpServerArgs),
}

/// aether-wire ipv4 client command
#[derive(Subcommand, Debug)]
pub enum Ipv4ClientCommand {
    Tcp(Ipv4TcpClientArgs),
    Udp(Ipv4UdpClientArgs),
}

/// TCP stream type server command
#[derive(Args, Clone, Debug)]
pub struct Ipv4TcpServerArgs {
    /// IPv4 address to bind to
    #[arg(short = 'b', long, default_value = "0.0.0.0")]
    pub bind: Ipv4Addr,

    /// port to listen on
    #[arg(short = 'p', long)]
    pub port: u16,

    /// exit after serving one benchmark session
    #[arg(long)]
    pub once: bool,
}

/// TCP stream type client command
#[derive(Args, Clone, Debug)]
pub struct Ipv4TcpClientArgs {
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
}

/// UDP stream type server command
#[derive(Args, Clone, Debug)]
pub struct Ipv4UdpServerArgs {
    /// IPv4 address to bind to (default: 0.0.0.0)
    #[arg(short = 'b', long, default_value = "0.0.0.0")]
    pub bind: Ipv4Addr,

    /// port to listen on
    #[arg(short = 'p', long)]
    pub port: u16,

    /// exit after serving one benchmark session
    #[arg(long)]
    pub once: bool,
}

/// UDP stream type client command
#[derive(Args, Clone, Debug)]
pub struct Ipv4UdpClientArgs {
    /// server IPv4 address to connect to
    #[arg(short = 's', long, value_parser = validate_server_ipv4)]
    pub server: Ipv4Addr,

    /// server port number
    #[arg(short = 'p', long)]
    pub port: u16,

    /// test duration (default: 10s)
    #[arg(short = 't', long, default_value = "10s", value_parser = parse_duration_min_1s)]
    pub time: Duration,

    /// number of parallel streams (default: 1, maximum: 128)
    #[arg(short = 'n', long, default_value_t = 1, value_parser = clap::value_parser!(u16).range(1..=128))]
    pub n_streams: u16,

    /// target bandwidth in bits per second (e.g. 10M, 1G)
    #[arg(short = 'b', long, value_parser = parse_bandwidth)]
    pub bandwidth: Option<u64>,

    /// UDP payload size in bytes (default: 1472 for standard MTU)
    #[arg(long, default_value_t = 1472, value_parser = clap::value_parser!(u16).range(10..=65507))]
    pub payload_size: u16,

    /// verify data integrity (reduces throughput, use for diagnostics)
    #[arg(long)]
    pub verify: bool,
}

/// checks for a minimal duration of 1s
fn parse_duration_min_1s(s: &str) -> Result<std::time::Duration, String> {
    let d = humantime::parse_duration(s).map_err(|e| e.to_string())?;

    if d < std::time::Duration::from_secs(1) {
        return Err("duration must be at least 1s".to_string());
    }

    Ok(d)
}

/// human-readable value parser for bandwidth
fn parse_bandwidth(s: &str) -> Result<u64, String> {
    let original = s;
    let s = s.trim().to_uppercase();

    if s.is_empty() {
        return Err("bandwidth cannot be empty".into());
    }

    let (num_part, multiplier) = if let Some(stripped) = s.strip_suffix('K') {
        (stripped, 1_000)
    } else if let Some(stripped) = s.strip_suffix('M') {
        (stripped, 1_000_000)
    } else if let Some(stripped) = s.strip_suffix('G') {
        (stripped, 1_000_000_000)
    } else {
        (s.as_str(), 1)
    };

    let value: u64 = num_part.parse().map_err(|_| {
        format!(
            "invalid bandwidth value: {}, expected formats like 10M, 1G, 500K",
            original
        )
    })?;

    if value == 0 {
        return Err("bandwidth must be > 0".into());
    }

    value
        .checked_mul(multiplier)
        .ok_or_else(|| format!("bandwidth too large: {}", original))
}

/// validates the provided server IPv4 is an actual reachable host (e.g., not a broadcast address)
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
