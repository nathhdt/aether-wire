//! check subcommand arguments and execution

use anyhow::Result;
use clap::Args;

use crate::check;
use crate::utils::system::host::ensure_root;

#[derive(Debug)]
pub struct CheckConfig {
    #[allow(unused)]
    pub iface: Option<String>,
}

impl From<CheckArgs> for CheckConfig {
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
}

impl CheckArgs {
    pub fn run(self) -> Result<()> {
        ensure_root();
        check::run(self.into())
    }
}
