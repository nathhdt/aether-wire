//! aether-wire qualify mode client

use anyhow::Result;

use crate::client::qualify::tcp_probe;

/// client qualify arguments structure
#[derive(Debug, Clone)]
pub struct QualifyParameters {
    pub server: std::net::Ipv4Addr,
    pub port: u16,
    pub export_json: bool,
}

/// runs the qualification pipeline
pub fn run(args: QualifyParameters) -> Result<()> {
    println!("[qualify] starting link qualification pipeline");
    println!("[qualify] target: {}:{}\n", args.server, args.port);

    // step 1: TCP probe
    tcp_probe::tcp_probe(args.server, args.port)?;

    // TODO: step 2: MTU sweep
    println!("\n[qualify] step 2: MTU sweep (not yet implemented)");

    // TODO: step 3: health check
    println!("\n[qualify] step 3: health check (not yet implemented)");

    // TODO: step 4: stress test
    println!("\nqualify] step 4: stress test (not yet implemented)");

    // TODO: step 5: report
    println!("\n[qualify] step 5: report (not yet implemented)");

    // TODO: step 6: diagnostic
    println!("[\nqualify] step 6: diagnostic (not yet implemented)");

    if args.export_json {
        println!("\n[qualify] JSON export will be implemented in step 6");
    }

    println!("\n[qualify] qualification pipeline complete");

    Ok(())
}
