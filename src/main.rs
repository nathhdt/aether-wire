//! aether-wire entrypoint

mod cli;
mod server;
mod udp;
mod utils;

use std::process::ExitCode;

fn main() -> ExitCode {
    if let Err(exit_code) = utils::system::host::ensure_root() {
        return exit_code;
    }

    cli::run();

    ExitCode::SUCCESS
}
