//! unified server handling all session types

pub mod tcp_handler;
pub mod udp_handler;

use anyhow::Result;
use std::net::{IpAddr, SocketAddr, TcpListener};

use crate::protocol::messages::{Hello, Message, PROTO_VERSION, SessionType};
use crate::protocol::wire;
use crate::{error, info, warn};

/// server parameters
pub struct ServerParameters {
    pub bind: std::net::Ipv4Addr,
    pub port: u16,
    pub once: bool,
}

/// runs the unified server
pub fn run(params: ServerParameters) -> Result<()> {
    let addr = SocketAddr::new(IpAddr::V4(params.bind), params.port);
    let listener = TcpListener::bind(addr)?;

    info!("server", "server listening on {addr}");

    loop {
        info!("server", "waiting for client...");

        let (mut ctrl_sock, ctrl_client) = listener.accept()?;

        // read hello to determine session type
        let hello: Hello = match wire::read_message(&mut ctrl_sock)? {
            Message::Hello(h) => h,
            other => {
                let _ = wire::send_message(
                    &mut ctrl_sock,
                    &Message::Error("expected hello message".into()),
                );
                error!("ctrl", "unexpected first message: {other:?}");
                continue;
            }
        };

        // version check
        if hello.version != PROTO_VERSION {
            let msg = format!(
                "incompatible version: client={}, server={}",
                hello.version, PROTO_VERSION
            );
            let _ = wire::send_message(&mut ctrl_sock, &Message::Error(msg.clone()));
            error!("ctrl", "{}", msg);
            continue;
        }

        // dispatch based on session type
        let session_result = match hello.session_type {
            SessionType::TcpBenchmark(config) => {
                tcp_handler::handle_tcp_session(ctrl_sock, ctrl_client, config, &params)
            }
            SessionType::UdpBenchmark(config) => {
                udp_handler::handle_udp_session(ctrl_sock, ctrl_client, config, &params)
            }
            SessionType::Qualify => {
                warn!(
                    "ctrl",
                    "client requested qualify mode (not yet implemented)"
                );
                wire::send_message(
                    &mut ctrl_sock,
                    &Message::Error("qualify mode not yet implemented".into()),
                )?;
                Ok(())
            }
        };

        if let Err(e) = session_result {
            error!("server", "session error: {e:#}");
        }

        if params.once {
            warn!("server", "--once flag set, exiting");
            break;
        }
    }

    Ok(())
}
