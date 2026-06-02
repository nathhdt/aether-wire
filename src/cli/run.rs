//! command line interface runner module

use anyhow::Result;
use clap::Parser;
use std::process::exit;

use crate::cli::Cli;

pub fn run() {
    if let Err(e) = try_run() {
        eprintln!("error: {e}");
        exit(1);
    }
}

fn try_run() -> Result<()> {
    let cli = Cli::try_parse().unwrap_or_else(|err| {
        eprint!("{}", err.to_string().replace("Usage:", "usage:"));
        exit(err.exit_code());
    });
    cli.run()
}
