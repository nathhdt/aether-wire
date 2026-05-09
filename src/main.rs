use anyhow::Result;
use clap::Parser;

mod cli;
mod client;
mod protocol;
mod server;
mod transport;
mod tui;
mod utils;

fn main() -> Result<()> {
    let cli = cli::Cli::parse();

    match cli.command {
        // server command
        cli::Command::Server(args) => {
            let server_params = server::tcp_handler::ServerParameters {
                bind: args.bind,
                port: args.port,
                once: args.once,
            };
            server::run(server_params)
        }

        // client command
        cli::Command::Client(client_cmd) => match client_cmd {
            cli::ClientCommand::Benchmark(args) => {
                args.validate()?;
                // CLI args to benchmark parameters
                let benchmark_parameters: client::benchmark::client::BenchmarkParameters =
                    client::benchmark::client::BenchmarkParameters {
                        server: args.server,
                        port: args.port,
                        duration: args.time,
                        n_streams: args.n_streams,
                        verify_integrity: args.verify,
                        direction: args.direction.to_direction(),
                    };
                client::benchmark::client::run(benchmark_parameters)
            }

            cli::ClientCommand::Qualify(args) => {
                // CLI args to qualify args
                let qualify_parameters = client::qualify::client::QualifyParameters {
                    server: args.server,
                    port: args.port,
                    export_json: args.json,
                };
                client::qualify::client::run(qualify_parameters)
            }
        },

        // TUI command
        cli::Command::Tui => tui::app::run(),
    }
}
