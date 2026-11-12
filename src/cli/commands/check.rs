//! check subcommand arguments and execution

use anyhow::Result;
use clap::{ArgAction, Args};

use crate::check;

#[derive(Debug)]
pub struct CheckCliArgs {
    pub iface: Option<String>,
}

impl From<CheckArgs> for CheckCliArgs {
    fn from(args: CheckArgs) -> Self {
        Self { iface: args.iface }
    }
}

#[derive(Args, Debug)]
#[command(help_template = "\
{about-with-newline}
usage: {usage}

options:
{options}
")]
pub struct CheckArgs {
    #[arg(
        short = 'i',
        long = "iface",
        value_name = "name",
        help = "network interface to check (default: all)"
    )]
    iface: Option<String>,

    #[arg(
        short = 'h',
        long = "help",
        action = ArgAction::Help,
        hide = true
    )]
    help: Option<bool>,
}

impl CheckArgs {
    pub fn run(self) -> Result<()> {
        super::ensure_root()?;
        check::run(self.into())
    }
}
