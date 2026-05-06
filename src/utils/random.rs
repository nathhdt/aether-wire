//! random generation utility functions

use rand_core::{OsRng, RngCore};

/// gives a random u64
pub fn rand_u64() -> u64 {
    OsRng.next_u64()
}
