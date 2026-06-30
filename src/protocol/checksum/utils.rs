//! utils checksum protocol module

/// adds 16-bit big-endian words in 'data' to 'acc'
pub fn sum16(data: &[u8], mut acc: u32) -> u32 {
    let mut chunks = data.chunks_exact(2);

    for chunk in &mut chunks {
        acc += u16::from_be_bytes([chunk[0], chunk[1]]) as u32;
    }

    if let [last] = chunks.remainder() {
        acc += u16::from_be_bytes([*last, 0]) as u32;
    }

    acc
}

/// folds 32-bit accumulator into checksum
pub fn fold(mut acc: u32) -> u16 {
    while acc >> 16 != 0 {
        acc = (acc & 0xFFFF) + (acc >> 16);
    }

    !(acc as u16)
}

/// finalizes UDP checksum
pub fn finalize_udp(acc: u32) -> u16 {
    match fold(acc) {
        0 => 0xFFFF,
        sum => sum,
    }
}
