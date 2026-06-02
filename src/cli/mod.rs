//! aether-wire command line interface

pub mod commands;
pub mod parsing;

mod run;

use anyhow::Result;
use clap::builder::Styles;
use clap::{CommandFactory, Parser};

pub use commands::Commands;
pub use run::run;

#[derive(Parser, Debug)]
#[command(
    name = "aw",
    about = "native linux E2E network performance and benchmarking tool",
    disable_help_flag = true,
    disable_help_subcommand = true,
    disable_version_flag = true,
    styles = Styles::plain(),
    help_template = "\
{about-with-newline}
usage: {usage}

commands:
{subcommands}

options:
{options}
"
)]
pub struct Cli {
    #[arg(short = 'v', long = "version", help = "print version")]
    version: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

impl Cli {
    pub fn run(self) -> Result<()> {
        if self.version {
            println!("aether-wire {}", env!("CARGO_PKG_VERSION"));
            return Ok(());
        }

        match self.command {
            Some(command) => command.run(),
            None => {
                Self::command().print_help()?;
                Ok(())
            }
        }
    }
}
