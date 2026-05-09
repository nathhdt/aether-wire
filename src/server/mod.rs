//! unified server handling all session types

use anyhow::Result;
use std::net::{IpAddr, SocketAddr};

use crate::cli::ServerArgs;

pub mod tcp_handler;

/// runs the server
pub fn run(args: ServerArgs) -> Result<()> {
    let addr = SocketAddr::new(IpAddr::V4(args.bind), args.port);

    println!("[server] server listening on {addr}");
    println!("[server] ready to handle sessions");

    tcp_handler::run_tcp_server(args)
}
