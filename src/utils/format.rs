//! format utilities module

/// converts bits per second data measure to human-readable format
pub fn human_bps(bps: f64) -> String {
    const K: f64 = 1_000.0;
    const M: f64 = 1_000_000.0;
    const G: f64 = 1_000_000_000.0;

    if bps >= G {
        format!("{:.2} Gbit/s", bps / G)
    } else if bps >= M {
        format!("{:.2} Mbit/s", bps / M)
    } else if bps >= K {
        format!("{:.2} kbit/s", bps / K)
    } else {
        format!("{bps:.0} bit/s")
    }
}
