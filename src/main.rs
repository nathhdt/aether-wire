//! aether-wire entrypoint

mod check;
mod cli;
mod server;
mod udp;
mod utils;

fn main() {
    cli::run();
}
