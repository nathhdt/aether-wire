//! check subcommand arguments and execution

use anyhow::Result;
use clap::Args;

use crate::check;
use crate::utils::system::host::ensure_root;

#[derive(Debug)]
pub struct CheckConfig {
    #[allow(dead_code)]
    pub iface: Option<String>,
    pub load_modules: bool,
}

impl From<CheckArgs> for CheckConfig {
    fn from(args: CheckArgs) -> Self {
        Self {
            iface: args.iface,
            load_modules: args.load_modules,
        }
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
        long = "load-modules",
        help = "attempt to load required kernel modules if not already loaded"
    )]
    load_modules: bool,
}

impl CheckArgs {
    pub fn run(self) -> Result<()> {
        ensure_root();
        check::run(self.into())
    }
}
