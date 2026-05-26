//! command line interface logic and subcommand definitions

use clap::{Args, Parser, Subcommand};
use std::net::Ipv4Addr;
use std::time::Duration;

use crate::protocol::messages::Direction;
use crate::utils::parser;

/// aether-wire base command
#[derive(Debug, Parser)]
#[command(
    name = env!("CARGO_BIN_NAME"),
    version,
    about = "native cross-platform E2E network performance and benchmarking tool",
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,
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

    /// UDP receiving buffer size
    #[arg(long = "udp-recv-buffer", default_value = "16M", value_parser = parser::parse_udp_buf_mem_size)]
    pub udp_recv_buffer: u64,

    /// exit after serving one session
    #[arg(long)]
    pub once: bool,
}

/// client subcommands
#[derive(Debug, Subcommand)]
pub enum ClientCommand {
    /// run a benchmark (TCP or UDP throughput measurement)
    #[command(subcommand)]
    Benchmark(BenchmarkCommand),

    /// run full link qualification pipeline
    Qualify(QualifyArgs),
}

/// benchmark subcommands
#[derive(Debug, Subcommand)]
pub enum BenchmarkCommand {
    /// TCP throughput benchmark
    Tcp(TcpBenchmarkArgs),

    /// UDP throughput benchmark
    Udp(UdpBenchmarkArgs),
}

/// TCP benchmark arguments
#[derive(Args, Clone, Debug)]
pub struct TcpBenchmarkArgs {
    /// server IPv4 address to connect to
    #[arg(short = 's', long, value_parser = parser::parse_server_ipv4)]
    pub server: Ipv4Addr,

    /// server port number
    #[arg(short = 'p', long)]
    pub port: u16,

    /// test duration (minimum 1s)
    #[arg(short = 't', long, default_value = "10s", value_parser = parser::parse_duration_min_1s)]
    pub time: Duration,

    /// number of parallel streams (1-32)
    #[arg(short = 'n', long, default_value_t = 1, value_parser = clap::value_parser!(u16).range(1..=32))]
    pub n_streams: u16,

    /// verify data integrity (reduces throughput, use for diagnostics)
    #[arg(long)]
    pub verify: bool,

    /// traffic direction mode
    #[command(flatten)]
    pub direction: DirectionArgs,
}

impl TcpBenchmarkArgs {
    pub fn validate(&self) -> anyhow::Result<()> {
        if self.verify && self.n_streams > 1 {
            anyhow::bail!("--verify can only be used with a single stream");
        }
        Ok(())
    }
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
    pub fn to_direction(&self) -> Direction {
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

/// UDP benchmark arguments
#[derive(Args, Clone, Debug)]
pub struct UdpBenchmarkArgs {
    /// server IPv4 address to connect to
    #[arg(short = 's', long, value_parser = parser::parse_server_ipv4)]
    pub server: Ipv4Addr,

    /// server port number
    #[arg(short = 'p', long)]
    pub port: u16,

    /// test duration (minimum 1s)
    #[arg(short = 't', long, default_value = "10s", value_parser = parser::parse_duration_min_1s)]
    pub time: Duration,

    /// number of parallel streams (depends on available CPU cores)
    #[arg(short = 'n', long, default_value_t = 1, value_parser = parser::parse_stream_number)]
    pub n_streams: u16,

    /// total target bandwidth (e.g., 1K, 1G, 50M)
    #[arg(short = 'b', long, value_parser = parser::parse_bandwidth)]
    pub bandwidth: u64,

    /// UDP payload size in bytes (e.g., 512, 1024, 1472)
    #[arg(short = 'l', long, default_value_t = 1400)]
    pub length: u16,
}

/// qualify mode arguments
#[derive(Args, Clone, Debug)]
pub struct QualifyArgs {
    /// server IPv4 address to connect to
    #[arg(short = 's', long, value_parser = parser::parse_server_ipv4)]
    pub server: Ipv4Addr,

    /// server port number
    #[arg(short = 'p', long)]
    pub port: u16,

    /// export full metrics to JSON file
    #[arg(long)]
    pub json: bool,
}
