//! collection of utility modules & functions

pub fn human_bps(bps: f64) -> String {
    const K: f64 = 1_000.0;
    const M: f64 = 1_000_000.0;
    const G: f64 = 1_000_000_000.0;

    if bps >= G {
        format!("{:.2} Gbit/s", bps / G)
    } else if bps >= M {
        format!("{:.2} Mbit/s", bps / M)
    } else if bps >= K {
        format!("{:.2} Kbit/s", bps / K)
    } else {
        format!("{bps:.0} bit/s")
    }
}

pub fn human_bytes(b: u64) -> String {
    const K: f64 = 1024.0;
    const M: f64 = 1024.0 * 1024.0;
    const G: f64 = 1024.0 * 1024.0 * 1024.0;

    let bf = b as f64;

    if bf >= G {
        format!("{:.2} GiB", bf / G)
    } else if bf >= M {
        format!("{:.2} MiB", bf / M)
    } else if bf >= K {
        format!("{:.2} KiB", bf / K)
    } else {
        format!("{b} B")
    }
}

pub fn rand_u64() -> u64 {
    use std::collections::hash_map::RandomState;
    use std::hash::{BuildHasher, Hasher};
    RandomState::new().build_hasher().finish()
}
