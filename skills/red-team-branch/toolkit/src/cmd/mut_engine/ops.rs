use super::bytes::{rd_u16, rd_u32, wr_u16, wr_u32};
use super::constants::*;
use super::Rng;

const XOR_BYTE_MAX: usize = u8::MAX as usize;

pub fn flip_bit(b: &mut [u8], rng: &mut Rng) {
    let bit = rng.below(b.len() * 8);
    b[bit / 8] ^= 1 << (bit % 8);
}

pub fn set_i8(b: &mut [u8], rng: &mut Rng) {
    let p = rng.below(b.len());
    b[p] = INTERESTING_8[rng.below(INTERESTING_8.len())] as u8;
}

pub fn set_i16(b: &mut [u8], rng: &mut Rng) {
    if b.len() >= 2 {
        let p = rng.below(b.len() - 1);
        wr_u16(
            b,
            p,
            INTERESTING_16[rng.below(INTERESTING_16.len())] as u16,
            rng.bool(),
        );
    }
}

pub fn set_i32(b: &mut [u8], rng: &mut Rng) {
    if b.len() >= 4 {
        let p = rng.below(b.len() - 3);
        wr_u32(
            b,
            p,
            INTERESTING_32[rng.below(INTERESTING_32.len())] as u32,
            rng.bool(),
        );
    }
}

pub fn add_sub_byte(b: &mut [u8], rng: &mut Rng, add: bool) {
    let p = rng.below(b.len());
    let n = 1 + rng.below(ARITH_MAX as usize) as u8;
    b[p] = if add {
        b[p].wrapping_add(n)
    } else {
        b[p].wrapping_sub(n)
    };
}

pub fn add_sub_word(b: &mut [u8], rng: &mut Rng, add: bool) {
    if b.len() < 2 {
        return;
    }
    let p = rng.below(b.len() - 1);
    let le = rng.bool();
    let n = 1 + rng.below(ARITH_MAX as usize) as u16;
    let v = if add {
        rd_u16(b, p, le).wrapping_add(n)
    } else {
        rd_u16(b, p, le).wrapping_sub(n)
    };
    wr_u16(b, p, v, le);
}

pub fn add_sub_dword(b: &mut [u8], rng: &mut Rng, add: bool) {
    if b.len() < 4 {
        return;
    }
    let p = rng.below(b.len() - 3);
    let le = rng.bool();
    let n = 1 + rng.below(ARITH_MAX as usize) as u32;
    let v = if add {
        rd_u32(b, p, le).wrapping_add(n)
    } else {
        rd_u32(b, p, le).wrapping_sub(n)
    };
    wr_u32(b, p, v, le);
}

pub fn xor_byte(b: &mut [u8], rng: &mut Rng) {
    let p = rng.below(b.len());
    b[p] ^= 1 + rng.below(XOR_BYTE_MAX) as u8;
}
