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
    const MAX_VERIFY_BUFFER: usize = 1024 * 1024 * 1024; // 1 GB hard limit

    let verified_gb = received.len() as f64 / MAX_VERIFY_BUFFER as f64;
    let total_gb = bytes_received as f64 / MAX_VERIFY_BUFFER as f64;

    if received.len() < bytes_received as usize {
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
    let verification_result: Result<()> = received
        .par_chunks(expected_len)
        .enumerate()
        .try_for_each(|(chunk_idx, chunk)| {
            let base_offset = chunk_idx * expected_len;

            // compare chunk against expected pattern
            if chunk != &expected[..chunk.len()] {
                // find exact mismatch byte
                for i in 0..chunk.len() {
                    if chunk[i] != expected[i] {
                        bail_error!(
                            "data",
                            "stream {}: integrity check failed at byte {}. expected 0x{:02x}, got 0x{:02x}",
                            stream_id,
                            base_offset + i,
                            expected[i],
                            chunk[i]
                        );
                    }
                }
            }

            Ok(())
        });

    verification_result?;
    info!("data", "stream {stream_id}: integrity check passed");

    Ok(())
}
