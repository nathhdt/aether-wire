//! aether-wire entrypoint

mod check;
mod cli;
mod protocol;
mod server;
mod udp;
mod utils;

fn main() {
    cli::run();
}
