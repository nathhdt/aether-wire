//! format utilities module

/// converts bits per second data measure to human-readable format
pub fn human_bps(bps: u64) -> String {
    const K: u64 = 1_000;
    const M: u64 = 1_000_000;
    const G: u64 = 1_000_000_000;

    if bps >= G {
        format!("{:.2} Gbit/s", bps as f64 / G as f64)
    } else if bps >= M {
        format!("{:.2} Mbit/s", bps as f64 / M as f64)
    } else if bps >= K {
        format!("{:.2} kbit/s", bps as f64 / K as f64)
    } else {
        format!("{bps} bit/s")
    }
}
