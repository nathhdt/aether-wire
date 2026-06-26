//! format utilities module

/// converts bits per second data measure to human-readable format
pub fn human_bps(bps: u64) -> String {
    const K: u64 = 1_000;
    const M: u64 = K * K;
    const G: u64 = K * M;
    const T: u64 = K * G;
    const P: u64 = K * T;
    const E: u64 = K * P;

    fn fmt(bps: u64, divisor: u64, unit: &str) -> String {
        if bps.is_multiple_of(divisor) {
            format!("{} {}", bps / divisor, unit)
        } else {
            format!("{:.2} {}", bps as f64 / divisor as f64, unit)
        }
    }

    if bps >= E {
        fmt(bps, E, "Ebit/s")
    } else if bps >= P {
        fmt(bps, P, "Pbit/s")
    } else if bps >= T {
        fmt(bps, T, "Tbit/s")
    } else if bps >= G {
        fmt(bps, G, "Gbit/s")
    } else if bps >= M {
        fmt(bps, M, "Mbit/s")
    } else if bps >= K {
        fmt(bps, K, "kbit/s")
    } else {
        format!("{bps} bit/s")
    }
}

/// converts bytes to human-readable binary format
pub fn human_bytes(bytes: u64) -> String {
    const KIB: u64 = 1024;
    const MIB: u64 = KIB * KIB;
    const GIB: u64 = KIB * MIB;
    const TIB: u64 = KIB * GIB;
    const PIB: u64 = KIB * TIB;
    const EIB: u64 = KIB * PIB;

    if bytes >= EIB {
        format!("{:.2} EiB", bytes as f64 / EIB as f64)
    } else if bytes >= PIB {
        format!("{:.2} PiB", bytes as f64 / PIB as f64)
    } else if bytes >= TIB {
        format!("{:.2} TiB", bytes as f64 / TIB as f64)
    } else if bytes >= GIB {
        format!("{:.2} GiB", bytes as f64 / GIB as f64)
    } else if bytes >= MIB {
        format!("{:.2} MiB", bytes as f64 / MIB as f64)
    } else if bytes >= KIB {
        format!("{:.2} KiB", bytes as f64 / KIB as f64)
    } else {
        format!("{bytes} B")
    }
}
