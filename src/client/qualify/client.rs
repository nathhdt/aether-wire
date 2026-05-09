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
    let vref = tcp_probe::tcp_probe(args.server, args.port)?;

    // TODO: step 2: MTU sweep
    println!("[qualify] step 2: MTU sweep (not yet implemented)");

    // TODO: step 3: health check
    println!("[qualify] step 3: health check (not yet implemented)");

    // TODO: step 4: stress test
    println!("[qualify] step 4: stress test (not yet implemented)");

    // TODO: step 5: report
    println!("[qualify] step 5: report (not yet implemented)");

    // TODO: step 6: diagnostic
    println!("[qualify] step 6: diagnostic (not yet implemented)");

    if args.export_json {
        println!("[qualify] JSON export will be implemented in step 6");
    }

    println!("\n[qualify] qualification pipeline complete");
    println!("[qualify] reference throughput (Vref): {:.2} Gbit/s", vref / 1_000_000_000.0);

    Ok(())
}
