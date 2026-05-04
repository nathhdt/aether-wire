use anyhow::Result;
use clap::Parser;

mod cli;
mod client;
mod payload;
mod proto;
mod server;
mod utils;
mod wire;

fn main() -> Result<()> {
    let cli = cli::Cli::parse();

    match cli.command {
        cli::Command::Serve(args) => server::run(args),
        cli::Command::Client(args) => client::run(args),
    }
}
