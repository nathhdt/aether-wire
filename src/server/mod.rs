//! unified server handling all session types

pub mod tcp;
pub mod udp;

use anyhow::Result;
use std::io::ErrorKind;
use std::net::{IpAddr, SocketAddr, TcpListener};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, mpsc};
use std::thread;
use std::time::Duration;

use crate::protocol::messages::{Hello, Message, PROTO_VERSION, SessionType};
use crate::protocol::stats::{TcpStreamStats, UdpStreamStats};
use crate::protocol::wire::{read_message, send_message};
use crate::{error, info, warn};

/// server parameters
pub struct ServerParameters {
    pub bind: std::net::Ipv4Addr,
    pub port: u16,
    pub once: bool,
}

/// events emitted by the TUI server
pub enum ServerTuiEvent {
    Listening {
        addr: String,
    },
    SessionStarted {
        peer: String,
        session_type: String,
    },
    TcpSessionResult {
        stats: Vec<TcpStreamStats>,
        is_sender: bool,
    },
    UdpSessionResult(Vec<UdpStreamStats>),
    SessionEnded {
        peer: String,
    },
    Error(String),
}

fn read_hello(ctrl_sock: &mut std::net::TcpStream) -> Result<Hello> {
    match read_message(ctrl_sock)? {
        Message::Hello(h) => Ok(h),
        other => {
            let _ = send_message(ctrl_sock, &Message::Error("expected hello message".into()));

            anyhow::bail!("unexpected first message: {other:?}");
        }
    }
}

fn validate_version(ctrl_sock: &mut std::net::TcpStream, version: u8) -> Result<()> {
    if version != PROTO_VERSION {
        let msg = format!(
            "incompatible version: client={}, server={}",
            version, PROTO_VERSION
        );

        let _ = send_message(ctrl_sock, &Message::Error(msg.clone()));

        anyhow::bail!(msg);
    }

    Ok(())
}

fn session_type_label(session_type: &SessionType) -> String {
    match session_type {
        SessionType::TcpBenchmark(_) => "TCP benchmark",
        SessionType::UdpBenchmark(_) => "UDP benchmark",
        SessionType::Qualify => "qualify",
    }
    .to_string()
}

fn handle_session(
    ctrl_sock: std::net::TcpStream,
    ctrl_client: SocketAddr,
    session_type: SessionType,
    params: &ServerParameters,
    tx: Option<mpsc::Sender<ServerTuiEvent>>,
) -> Result<()> {
    match session_type {
        SessionType::TcpBenchmark(config) => {
            tcp::handler::handle_tcp_session(ctrl_sock, ctrl_client, config, params, tx)
        }

        SessionType::UdpBenchmark(config) => {
            udp::handler::handle_udp_session(ctrl_sock, ctrl_client, config, params, tx)
        }

        SessionType::Qualify => {
            warn!(
                "ctrl",
                "client requested qualify mode (not yet implemented)"
            );

            let mut ctrl_sock = ctrl_sock;

            send_message(
                &mut ctrl_sock,
                &Message::Error("qualify mode not yet implemented".into()),
            )?;

            Ok(())
        }
    }
}

/// runs the CLI server
pub fn run(params: ServerParameters) -> Result<()> {
    let addr = SocketAddr::new(IpAddr::V4(params.bind), params.port);
    let listener = TcpListener::bind(addr)?;

    info!("server", "server listening on {addr}");

    loop {
        info!("server", "waiting for client...");

        let (mut ctrl_sock, ctrl_client) = listener.accept()?;

        // read hello to determine session type
        let hello = match read_hello(&mut ctrl_sock) {
            Ok(h) => h,
            Err(e) => {
                error!("ctrl", "{e:#}");
                continue;
            }
        };

        // version check
        if let Err(e) = validate_version(&mut ctrl_sock, hello.version) {
            error!("ctrl", "{e:#}");
            continue;
        }

        // dispatch based on session type
        if let Err(e) = handle_session(ctrl_sock, ctrl_client, hello.session_type, &params, None) {
            error!("server", "session error: {e:#}");
        }

        if params.once {
            warn!("server", "--once flag set, exiting");
            break;
        }
    }

    Ok(())
}

/// runs the server for TUI use
pub fn run_tui(
    params: ServerParameters,
    tx: mpsc::Sender<ServerTuiEvent>,
    stop: Arc<AtomicBool>,
) -> Result<()> {
    let addr = SocketAddr::new(IpAddr::V4(params.bind), params.port);
    let listener = TcpListener::bind(addr)?;
    listener.set_nonblocking(true)?;

    let _ = tx.send(ServerTuiEvent::Listening {
        addr: addr.to_string(),
    });

    loop {
        if stop.load(Ordering::Relaxed) {
            break;
        }

        match listener.accept() {
            Ok((mut ctrl_sock, ctrl_client)) => {
                // session handling uses blocking I/O
                ctrl_sock.set_nonblocking(false)?;

                // read hello
                let hello = match read_hello(&mut ctrl_sock) {
                    Ok(h) => h,
                    Err(e) => {
                        let _ = tx.send(ServerTuiEvent::Error(format!("{ctrl_client}: {e:#}")));

                        continue;
                    }
                };

                // version check
                if let Err(e) = validate_version(&mut ctrl_sock, hello.version) {
                    let _ = tx.send(ServerTuiEvent::Error(format!("{ctrl_client}: {e:#}")));

                    continue;
                }

                let session_type = session_type_label(&hello.session_type);

                let _ = tx.send(ServerTuiEvent::SessionStarted {
                    peer: ctrl_client.to_string(),
                    session_type: session_type.clone(),
                });

                let result = handle_session(
                    ctrl_sock,
                    ctrl_client,
                    hello.session_type,
                    &params,
                    Some(tx.clone()),
                );

                match result {
                    Ok(()) => {
                        let _ = tx.send(ServerTuiEvent::SessionEnded {
                            peer: ctrl_client.to_string(),
                        });
                    }

                    Err(e) => {
                        let _ = tx.send(ServerTuiEvent::Error(format!(
                            "session error ({ctrl_client}): {e:#}"
                        )));
                    }
                }

                if params.once {
                    break;
                }
            }

            // no pending connection
            Err(e) if e.kind() == ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(50));
            }

            Err(e) => {
                let _ = tx.send(ServerTuiEvent::Error(format!("accept error: {e}")));
                break;
            }
        }
    }

    Ok(())
}
