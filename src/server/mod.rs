//! unified server handling all session types

use anyhow::Result;
use std::net::{IpAddr, SocketAddr};

use crate::info;
use crate::server::tcp_handler::ServerParameters;

pub mod tcp_handler;

/// runs the server
pub fn run(params: ServerParameters) -> Result<()> {
    let addr = SocketAddr::new(IpAddr::V4(params.bind), params.port);

    info!("server", "server listening on {addr}");
    info!("server", "ready to handle sessions");

    tcp_handler::run_tcp_server(params)
}
