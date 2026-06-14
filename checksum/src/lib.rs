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
