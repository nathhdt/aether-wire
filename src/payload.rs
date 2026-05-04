//! payload generation utilities

use rand_chacha::ChaCha8Rng;
use rand_core::{RngCore, SeedableRng};

// reusable payload buffer size
pub const BUFFER_SIZE: usize = 256 * 1024;

/// generates buffer data
pub fn make_buffer(seed: u64) -> Vec<u8> {
    // buffer specifications
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let mut buf = vec![0u8; BUFFER_SIZE];

    // buffer data generation
    rng.fill_bytes(&mut buf);

    buf
}

/// derives a per-stream seed from a session seed
#[inline]
pub fn stream_seed(session_seed: u64, stream_id: u16) -> u64 {
    // stream seed generation using golden ratio constant for good bit mixing
    session_seed ^ (stream_id as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15)
}
