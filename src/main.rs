use anyhow::Result;
use clap::Parser;

mod cli;
mod client;
mod protocol;
mod server;
mod transport;
mod utils;

fn main() -> Result<()> {
    let cli = cli::Cli::parse();

    match cli.command {
        cli::Command::Server(args) => {
            // CLI args to server args
            let server_args = cli::ServerArgs {
                bind: args.bind,
                port: args.port,
                once: args.once,
            };
            server::run(server_args)
        }

        cli::Command::Client(client_cmd) => match client_cmd {
            cli::ClientCommand::Benchmark(args) => {
                // CLI args to benchmark args
                let benchmark_args = client::benchmark::client::BenchmarkArgs {
                    server: args.server,
                    port: args.port,
                    duration: args.time,
                    n_streams: args.n_streams,
                    verify_integrity: args.verify,
                    direction: args.direction.to_direction(),
                };
                client::benchmark::client::run(benchmark_args)
            }

            cli::ClientCommand::Qualify(args) => {
                // CLI args to qualify args
                let qualify_args = client::qualify::client::QualifyArgs {
                    server: args.server,
                    port: args.port,
                    export_json: args.json,
                };
                client::qualify::client::run(qualify_args)
            }
        },
    }
}
