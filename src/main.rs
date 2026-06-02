//! aether-wire entrypoint

mod cli;
mod server;
mod udp;
mod utils;

fn main() {
    cli::run();
}
