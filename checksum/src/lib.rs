#![no_std]
use byteorder::{ByteOrder, NetworkEndian};

const fn propagate_carries(word: u32) -> u16 {
    let sum = (word >> 16) + (word & 0xffff);
    ((sum >> 16) as u16) + (sum as u16)
}

/// Compute an RFC 1071 compliant checksum (without the final complement).
#[inline(never)]
pub fn checksum_original(mut data: &[u8]) -> u16 {
    let mut accum = 0;

    // For each 32-byte chunk...
    const CHUNK_SIZE: usize = 32;
    while data.len() >= CHUNK_SIZE {
        let mut d = &data[..CHUNK_SIZE];
        // ... take by 2 bytes and sum them.
        while d.len() >= 2 {
            accum += NetworkEndian::read_u16(d) as u32;
            d = &d[2..];
        }

        data = &data[CHUNK_SIZE..];
    }

    // Sum the rest that does not fit the last 32-byte chunk,
    // taking by 2 bytes.
    while data.len() >= 2 {
        accum += NetworkEndian::read_u16(data) as u32;
        data = &data[2..];
    }

    // Add the last remaining odd byte, if any.
    if let Some(&value) = data.first() {
        accum += (value as u32) << 8;
    }

    propagate_carries(accum)
}

#[inline(never)]
pub fn checksum_indexed(mut data: &[u8]) -> u16 {
    let mut accum = 0;

    // For each 32-byte chunk...
    const CHUNK_SIZE: usize = 32;
    while data.len() >= CHUNK_SIZE {
        let chunk = &data[..CHUNK_SIZE];
        let mut i = 0;
        // ... take by 2 bytes and sum them.
        while i + 1 < CHUNK_SIZE {
            accum += u16::from_be_bytes([chunk[i], chunk[i + 1]]) as u32;
            i += 2;
        }

        data = &data[CHUNK_SIZE..];
    }

    // Sum the rest that does not fit the last 32-byte chunk,
    // taking by 2 bytes.
    let mut i = 0;
    while i + 1 < data.len() {
        accum += u16::from_be_bytes([data[i], data[i + 1]]) as u32;
        i += 2;
    }

    // Add the last remaining odd byte, if any.
    if i < data.len() {
        accum += (data[i] as u32) << 8;
    }

    propagate_carries(accum)
}

#[inline(never)]
pub fn checksum_chunks_exact_no_bigchunk(data: &[u8]) -> u16 {
    let mut accum = 0;

    // ... take by 2 bytes and sum them.
    let mut chunks = data.chunks_exact(2);
    for pair in &mut chunks {
        accum += u16::from_be_bytes([pair[0], pair[1]]) as u32;
    }

    // Add the last remaining odd byte, if any.
    if let Some(&byte) = chunks.remainder().first() {
        accum += (byte as u32) << 8;
    }

    propagate_carries(accum)
}

#[inline]
pub fn checksum_chunks_exact(data: &[u8]) -> u16 {
    let mut accum = 0;

    // For each 32-byte chunk...
    const CHUNK_SIZE: usize = 32;
    const WORD_SIZE: usize = 2;
    let mut chunks = data.chunks_exact(CHUNK_SIZE);
    for chunk in &mut chunks {
        // ... take by 2 bytes and sum them.
        for pair in chunk.chunks_exact(WORD_SIZE) {
            accum += u16::from_be_bytes([pair[0], pair[1]]) as u32;
        }
    }

    // Sum the rest that does not fit the last 32-byte chunk,
    // taking by 2 bytes.
    let remainder = chunks.remainder();
    let mut word_pairs = remainder.chunks_exact(WORD_SIZE);
    for pair in &mut word_pairs {
        accum += u16::from_be_bytes([pair[0], pair[1]]) as u32;
    }

    // Add the last remaining odd byte, if any.
    if let Some(&byte) = word_pairs.remainder().first() {
        accum += (byte as u32) << 8;
    }

    propagate_carries(accum)
}

#[inline(always)]
const fn add_assign_with_carry_u32(a: &mut u32, b: u32) {
    // should inline to the adc instruction on x86 and arm
    let carry;
    (*a, carry) = a.overflowing_add(b);
    // carry only non-zero when we just overflowed,
    // adding this will not trigger another overflow
    *a += carry as u32;
}

#[inline(always)]
fn ne_read_u16(buf: &[u8]) -> u16 {
    u16::from_ne_bytes(buf[..2].try_into().unwrap())
}

#[inline(always)]
fn ne_read_u32(buf: &[u8]) -> u32 {
    u32::from_ne_bytes(buf[..4].try_into().unwrap())
}

#[inline(never)]
pub fn checksum_sliced_ne(mut data: &[u8]) -> u16 {
    let mut accum: u32 = 0;

    // Sum as much as possible in 4 byte chunks
    const CHUNK_SIZE: usize = 4;
    while data.len() >= CHUNK_SIZE {
        add_assign_with_carry_u32(&mut accum, ne_read_u32(data));

        data = &data[CHUNK_SIZE..];
    }

    // Sum the rest that does not fit the last 32-byte chunk,
    // taking by 2 bytes.
    while data.len() >= 2 {
        add_assign_with_carry_u32(&mut accum, ne_read_u16(data) as u32);
        data = &data[2..];
    }

    // Add the last remaining odd byte, if any.
    if let Some(&value) = data.first() {
        add_assign_with_carry_u32(&mut accum, ne_read_u16(&[value, 0]) as u32);
    }

    let val = propagate_carries(accum);
    u16::from_be_bytes(val.to_ne_bytes())
}

#[inline(never)]
pub fn checksum_sliced_ne_sep(mut data: &[u8]) -> u16 {
    let mut accum: u32 = 0;
    let mut accum_carry: u32 = 0;

    // Sum as much as possible in 4 byte chunks
    const CHUNK_SIZE: usize = 4;
    while data.len() >= CHUNK_SIZE {
        let val = ne_read_u32(data);
        let carry;
        (accum, carry) = accum.overflowing_add(val);
        accum_carry += carry as u32;

        data = &data[CHUNK_SIZE..];
    }

    add_assign_with_carry_u32(&mut accum, accum_carry);

    // Sum the rest that does not fit the last 32-byte chunk,
    // taking by 2 bytes.
    if data.len() >= 2 {
        add_assign_with_carry_u32(&mut accum, ne_read_u16(data) as u32);
        data = &data[2..];
    }

    // Add the last remaining odd byte, if any.
    if let Some(&value) = data.first() {
        add_assign_with_carry_u32(&mut accum, ne_read_u16(&[value, 0]) as u32);
    }

    let val = propagate_carries(accum);
    u16::from_be_bytes(val.to_ne_bytes())
}

#[inline(never)]
pub fn checksum_sliced_ne_sep_unroll(mut data: &[u8]) -> u16 {
    let mut accum: u32 = 0;

    let mut accum2: u32 = 0;
    let mut accum_carry: u32 = 0;
    let mut accum_carry2: u32 = 0;

    // Sum as much as possible in 4 byte chunks
    const CHUNK_SIZE: usize = 8;
    while data.len() >= CHUNK_SIZE {
        let val = ne_read_u32(data);
        let val2 = ne_read_u32(&data[4..]);

        let carry;
        let carry2;

        (accum, carry) = accum.overflowing_add(val);
        accum_carry += carry as u32;

        (accum2, carry2) = accum2.overflowing_add(val2);
        accum_carry2 += carry2 as u32;

        data = &data[CHUNK_SIZE..];
    }

    add_assign_with_carry_u32(&mut accum2, accum_carry2);
    add_assign_with_carry_u32(&mut accum, accum_carry);
    add_assign_with_carry_u32(&mut accum, accum2);

    // Sum the rest that does not fit the last 32-byte chunk,
    // taking by 2 bytes.
    while data.len() >= 2 {
        add_assign_with_carry_u32(&mut accum, ne_read_u16(data) as u32);
        data = &data[2..];
    }

    // Add the last remaining odd byte, if any.
    if let Some(&value) = data.first() {
        add_assign_with_carry_u32(&mut accum, ne_read_u16(&[value, 0]) as u32);
    }

    let val = propagate_carries(accum);
    u16::from_be_bytes(val.to_ne_bytes())
}

#[inline(never)]
pub fn checksum_chunks_ne_sep(mut data: &[u8]) -> u16 {
    let mut accum: u32 = 0;
    let mut accum_carry: u32 = 0;

    // Sum as much as possible in 4 byte chunks
    let chunks;
    (chunks, data) = data.as_chunks::<4>();
    for chunk in chunks {
        let val = ne_read_u32(chunk);
        let carry;
        (accum, carry) = accum.overflowing_add(val);
        accum_carry += carry as u32;
    }

    add_assign_with_carry_u32(&mut accum, accum_carry);

    // Sum the rest that does not fit the last 32-byte chunk,
    // taking by 2 bytes.
    if data.len() >= 2 {
        add_assign_with_carry_u32(&mut accum, ne_read_u16(data) as u32);
        data = &data[2..];
    }

    // Add the last remaining odd byte, if any.
    if let Some(&value) = data.first() {
        add_assign_with_carry_u32(&mut accum, ne_read_u16(&[value, 0]) as u32);
    }

    let val = propagate_carries(accum);
    u16::from_be_bytes(val.to_ne_bytes())
}

#[inline(never)]
pub fn checksum_sliced_ne_u16(mut data: &[u8]) -> u16 {
    let mut accum: u32 = 0;

    // Sum as much as possible in 2 byte chunks
    const CHUNK_SIZE: usize = 2;
    while data.len() >= CHUNK_SIZE {
        accum += ne_read_u16(data) as u32;

        data = &data[CHUNK_SIZE..];
    }

    // Add the last remaining odd byte, if any.
    if let Some(&value) = data.first() {
        add_assign_with_carry_u32(&mut accum, ne_read_u16(&[value, 0]) as u32);
    }

    let val = propagate_carries(accum);
    u16::from_be_bytes(val.to_ne_bytes())
}

#[inline(never)]
pub fn checksum_sliced_ne_u16_unroll(mut data: &[u8]) -> u16 {
    // checksum_sliced_ne_u16_unroll
    let mut accum: u32 = 0;
    let mut accum2: u32 = 0;

    // Sum as much as possible in 4 byte chunks
    const CHUNK_SIZE: usize = 4;
    while data.len() >= CHUNK_SIZE {
        accum += u16::from_ne_bytes([data[0], data[1]]) as u32;
        accum2 += u16::from_ne_bytes([data[2], data[3]]) as u32;

        data = &data[CHUNK_SIZE..];
    }

    accum += accum2;

    if data.len() >= 2 {
        accum += u16::from_ne_bytes([data[0], data[1]]) as u32;
        data = &data[2..];
    }

    // Add the last remaining odd byte, if any.
    if let Some(&byte) = data.first() {
        // accum += u16::from_ne_bytes([byte, 0]) as u32;
        accum += u16::from_ne_bytes([byte, 0]) as u32;
    }

    let val = propagate_carries(accum);
    u16::from_be_bytes(val.to_ne_bytes())
}

#[inline(never)]
pub fn checksum_chunks_ne_u16(data: &[u8]) -> u16 {
    let mut accum: u32 = 0;

    // ... take by 2 bytes and sum them.
    let mut chunks = data.chunks_exact(2);
    for pair in &mut chunks {
        accum += u16::from_ne_bytes([pair[0], pair[1]]) as u32;
    }

    // Add the last remaining odd byte, if any.
    if let Some(&byte) = chunks.remainder().first() {
        accum += u16::from_ne_bytes([byte, 0]) as u32;
    }

    let val = propagate_carries(accum);
    u16::from_be_bytes(val.to_ne_bytes())
}

#[inline(never)]
pub fn checksum_chunks_ne_u16_unroll(data: &[u8]) -> u16 {
    let mut accum: u32 = 0;
    let mut accum2: u32 = 0;

    // ... take by 2 bytes and sum them.
    let mut chunks = data.chunks_exact(4);
    for pair in &mut chunks {
        accum += u16::from_ne_bytes([pair[0], pair[1]]) as u32;
        accum2 += u16::from_ne_bytes([pair[2], pair[3]]) as u32;
    }

    accum += accum2;

    let mut remainder = chunks.remainder();
    if remainder.len() >= 2 {
        accum += u16::from_ne_bytes([remainder[0], remainder[1]]) as u32;
        remainder = &remainder[2..];
    }

    // Add the last remaining odd byte, if any.
    if let Some(&byte) = remainder.first() {
        accum += u16::from_ne_bytes([byte, 0]) as u32;
    }

    let val = propagate_carries(accum);
    u16::from_be_bytes(val.to_ne_bytes())
}

#[inline(never)]
pub fn checksum_wide(data: &[u8]) -> u16 {
    // inspired by: https://stackoverflow.com/questions/78889987/how-to-perform-parallel-addition-using-avx-with-carry-overflow-fed-back-into-t
    // let mut accum: u32 = 0;
    let mut wide_accum = wide::u32x4::ZERO;
    let mut wide_carry_accum = wide::u32x4::ZERO;

    let (chunks, tail) = data.as_chunks::<16>();
    for chunk in chunks {
        // let u16s: &[u32; 4] = wide::bytemuck::must_cast_ref(chunk);
        // let vals = wide::u32x4::from(*u16s);
        let vals: wide::u32x4 = bytemuck::pod_read_unaligned(chunk);
        let saturated_add = wide_accum.saturating_add(vals);
        wide_accum += vals;

        // if there was overflow then saturated add will not be equal to wrapping add
        // equal mask will take max value for each non-equal lane
        let equal_mask = wide_accum.simd_ne(saturated_add);
        wide_carry_accum -= equal_mask;
    }

    // add the carries in, checking for more carries
    let saturated_add = wide_accum.saturating_add(wide_carry_accum);
    wide_accum += wide_carry_accum;
    let equal_mask = wide_accum.simd_ne(saturated_add);
    wide_accum -= equal_mask;

    // collapse wide vec into u32
    let mut accum: u32 = 0;
    let vals: [u32; 4] = wide_accum.into();
    for val in vals {
        // TODO: maybe try SIMD version of propagate_carries to collapse vector?
        add_assign_with_carry_u32(&mut accum, val);
    }

    // we can add without carry into tail_accum u32 as tail is guaranteed to be short
    let mut tail_accum: u32 = 0;

    // ... take by 2 bytes and sum them.
    let (chunks, tail) = tail.as_chunks::<2>();
    for pair in chunks {
        // tail_accum += u16::from_ne_bytes([pair[0], pair[1]]) as u32;
        tail_accum += u16::from_ne_bytes(*pair) as u32;
    }

    // Add the last remaining odd byte, if any.
    if let Some(&byte) = tail.first() {
        tail_accum += u16::from_ne_bytes([byte, 0]) as u32;
    }

    add_assign_with_carry_u32(&mut accum, tail_accum);
    let val = propagate_carries(accum);
    u16::from_be_bytes(val.to_ne_bytes())
}

#[cfg(test)]
mod tests {
    use wide::u32x4;

    use super::*;

    /// compare all checksum implementations against original
    /// for given data
    fn check_checksums(data: &[u8]) {
        let res_orig = checksum_original(data);

        assert_eq!(checksum_wide(data), res_orig);
        assert_eq!(checksum_indexed(data), res_orig);
        assert_eq!(checksum_chunks_exact_no_bigchunk(data), res_orig);
        assert_eq!(checksum_chunks_exact(data), res_orig);
        assert_eq!(checksum_sliced_ne(data), res_orig);
        assert_eq!(checksum_sliced_ne_sep(data), res_orig);
        assert_eq!(checksum_sliced_ne_sep_unroll(data), res_orig);
        assert_eq!(checksum_chunks_ne_sep(data), res_orig);
        assert_eq!(checksum_sliced_ne_u16(data), res_orig);
        assert_eq!(checksum_sliced_ne_u16_unroll(data), res_orig);
        assert_eq!(checksum_chunks_ne_u16(data), res_orig);
        assert_eq!(checksum_chunks_ne_u16_unroll(data), res_orig);
        assert_eq!(checksum_wide(data), res_orig);
    }

    #[test]
    fn test_32_1() {
        let data = [254; 32];
        check_checksums(&data);
    }

    #[test]
    fn test_33_1() {
        let data = [254; 33];
        check_checksums(&data);
    }

    #[test]
    fn test_35_1() {
        let data = [254; 35];
        check_checksums(&data);
    }

    #[test]
    fn test_37_1() {
        let data = [254; 37];
        check_checksums(&data);
    }

    #[test]
    fn test_1024() {
        let data = [254; 1024];
        check_checksums(&data);
    }

    #[test]
    fn test_simd_ne() {
        let a = u32x4::splat(2);
        let b = u32x4::from([0, 2, 2, 4]);

        let res = a.simd_ne(b);
        let expect = u32x4::from([u32::MAX, 0, 0, u32::MAX]);

        assert_eq!(res, expect);
    }
}
