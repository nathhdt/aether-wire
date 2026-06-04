//! command line interface commands

pub mod check;
pub mod server;
pub mod udp;

use anyhow::Result;
use clap::Subcommand;

use check::CheckArgs;
use server::ServerArgs;
use udp::UdpArgs;

#[derive(Subcommand, Debug)]
pub enum Commands {
    #[command(about = "check system and interface XDP compatibility")]
    Check(CheckArgs),

    #[command(about = "run a UDP performance test")]
    Udp(UdpArgs),

    #[command(about = "run a benchmark server")]
    Server(ServerArgs),
}

impl Commands {
    pub fn run(self) -> Result<()> {
        match self {
            Self::Check(args) => args.run(),
            Self::Udp(args) => args.run(),
            Self::Server(args) => args.run(),
        }
    }
}
