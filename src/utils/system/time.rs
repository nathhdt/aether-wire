//! monotonic time module

use rustix::time::{ClockId, clock_gettime};

/// returns current CLOCK_MONOTONIC
#[allow(dead_code)]
pub fn monotonic_now_ns() -> u64 {
    let ts = clock_gettime(ClockId::Monotonic);

    (ts.tv_sec as u64) * 1_000_000_000 + ts.tv_nsec as u64
}
