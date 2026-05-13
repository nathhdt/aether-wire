//! aether-wire qualify mode client

use anyhow::Result;
use std::net::Ipv4Addr;

use crate::client::qualify::tcp_probe;
use crate::{info, warn};

/// client qualify arguments structure
#[derive(Debug, Clone)]
pub struct QualifyParameters {
    pub server: Ipv4Addr,
    pub port: u16,
    pub export_json: bool,
}

/// runs the qualification pipeline
pub fn run(args: QualifyParameters) -> Result<()> {
    info!("qualify", "starting link qualification pipeline");
    info!("qualify", "target: {}:{}", args.server, args.port);

    // step 1: TCP probe
    tcp_probe::tcp_probe(args.server, args.port)?;

    // TODO: step 2: MTU sweep
    warn!("qualify", "step 2: MTU sweep (not yet implemented)");

    // TODO: step 3: health check
    warn!("qualify", "step 3: health check (not yet implemented)");

    // TODO: step 4: stress test
    warn!("qualify", "step 4: stress test (not yet implemented)");

    // TODO: step 5: report
    warn!("qualify", "step 5: report (not yet implemented)");

    // TODO: step 6: diagnostic
    warn!("qualify", "step 6: diagnostic (not yet implemented)");

    if args.export_json {
        info!("qualify", "JSON export will be implemented in step 6");
    }

    info!("qualify", "qualification pipeline complete");

    Ok(())
}
