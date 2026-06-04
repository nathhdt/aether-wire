//! aether-wire entrypoint

mod check;
mod cli;
mod server;
mod udp;
mod utils;

fn main() {
    utils::system::host::ensure_root();

    cli::run();
}
