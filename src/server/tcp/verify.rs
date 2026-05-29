//! TCP payload verification module

use anyhow::Result;
use rayon::prelude::*;

use crate::utils::payload::{make_buffer, stream_seed};
use crate::{bail_error, info, warn};

/// verifies received TCP stream payload integrity
pub fn verify_received_data(
    stream_id: u16,
    session_seed: u64,
    bytes_received: u64,
    received: Vec<u8>,
) -> Result<()> {
    const GIB: f64 = 1024.0 * 1024.0 * 1024.0;

    let verified_bytes = received.len() as u64;
    let verified_gb = verified_bytes as f64 / GIB;
    let total_gb = bytes_received as f64 / GIB;

    if verified_bytes < bytes_received {
        warn!(
            "data",
            "stream {stream_id}: verifying first {verified_gb:.2} GiB of {total_gb:.2} GiB total..."
        );
    } else {
        warn!(
            "data",
            "stream {stream_id}: verifying {verified_gb:.2} GiB..."
        );
    }

    let expected = make_buffer(stream_seed(session_seed, stream_id));
    let expected_len = expected.len();

    // parallel verification by chunks
    received
        .par_chunks(expected_len)
        .enumerate()
        .try_for_each(|(chunk_idx, chunk)| {
            let base_offset = chunk_idx * expected_len;

            // compare chunk against expected pattern
            if chunk != &expected[..chunk.len()] {
                // find exact mismatch byte
                for (i, (&got, &exp)) in chunk.iter().zip(expected.iter()).enumerate() {
                    if got != exp {
                        bail_error!(
                            "data",
                            "stream {}: integrity check failed at byte {}. expected 0x{:02x}, got 0x{:02x}",
                            stream_id,
                            base_offset + i,
                            exp,
                            got
                        );
                    }
                }
            }

            Ok(())
        })?;

    info!("data", "stream {stream_id}: integrity check passed");

    Ok(())
}
