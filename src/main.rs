use anyhow::Result;
use clap::Parser;

mod cli;
mod client;
mod protocol;
mod server;
mod socket;
mod tui;
mod utils;

fn main() -> Result<()> {
    let cli = cli::Cli::parse();

    match cli.command {
        // server command
        Some(cli::Command::Server(args)) => {
            let server_params = server::ServerParameters {
                bind: args.bind,
                port: args.port,
                udp_recv_buffer: args.udp_recv_buffer,
                once: args.once,
            };
            server::run(server_params)
        }

        // TCP benchmark command
        Some(cli::Command::Tcp(args)) => {
            args.validate()?;

            // CLI args to TCP benchmark parameters
            let tcp_benchmark_parameters = client::benchmark::TcpBenchmarkParameters {
                server: args.server,
                port: args.port,
                duration: args.time,
                n_streams: args.n_streams,
                verify_integrity: args.verify,
                direction: args.direction.to_direction(),
            };

            client::benchmark::client::run_tcp(tcp_benchmark_parameters)
        }

        // UDP benchmark command
        Some(cli::Command::Udp(args)) => {
            // CLI args to UDP benchmark parameters
            let udp_benchmark_parameters = client::benchmark::UdpBenchmarkParameters {
                server: args.server,
                port: args.port,
                duration: args.time,
                n_streams: args.n_streams,
                bandwidth: args.bandwidth,
                payload_size: args.length,
            };

            client::benchmark::client::run_udp(udp_benchmark_parameters)
        }

        // qualify command
        Some(cli::Command::Qualify(args)) => {
            // CLI args to qualify args
            let qualify_parameters = client::qualify::QualifyParameters {
                server: args.server,
                port: args.port,
                export_json: args.json,
            };

            client::qualify::client::run(qualify_parameters)
        }

        // default to TUI
        None => tui::runner::run(),
    }
}
