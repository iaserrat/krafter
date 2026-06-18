use super::blocks::{delete_block, insert_block, overwrite_block};
use super::constants::*;
use super::dict::{dict_insert, dict_overwrite};
use super::havoc_op::HavocOp;
use super::ops::*;
use super::Rng;

const MIN_STACK_POWER: usize = 1;
const EMPTY_SEED_BYTES: usize = 1;

pub fn havoc(seed: &[u8], dict: &[Vec<u8>], rng: &mut Rng) -> Vec<u8> {
    let mut b = seed_vec(seed, rng);
    for _ in 0..(1usize << (MIN_STACK_POWER + rng.below(HAVOC_STACK_POW2))) {
        let len = b.len();
        if len == 0 {
            break;
        }
        match HavocOp::pick(rng, !dict.is_empty()) {
            HavocOp::FlipBit => flip_bit(&mut b, rng),
            HavocOp::SetI8 => set_i8(&mut b, rng),
            HavocOp::SetI16 => set_i16(&mut b, rng),
            HavocOp::SetI32 => set_i32(&mut b, rng),
            HavocOp::SubByte => add_sub_byte(&mut b, rng, false),
            HavocOp::AddByte => add_sub_byte(&mut b, rng, true),
            HavocOp::SubWord => add_sub_word(&mut b, rng, false),
            HavocOp::AddWord => add_sub_word(&mut b, rng, true),
            HavocOp::SubDword => add_sub_dword(&mut b, rng, false),
            HavocOp::AddDword => add_sub_dword(&mut b, rng, true),
            HavocOp::XorByte => xor_byte(&mut b, rng),
            HavocOp::DeleteBlock => delete_block(&mut b, rng),
            HavocOp::InsertBlock => insert_block(&mut b, rng),
            HavocOp::OverwriteBlock => overwrite_block(&mut b, rng),
            HavocOp::DictOverwrite => dict_overwrite(&mut b, dict, rng),
            HavocOp::DictInsert => dict_insert(&mut b, dict, rng),
        }
        b.truncate(b.len().min(MAX_FILE));
    }
    b
}

fn seed_vec(seed: &[u8], rng: &mut Rng) -> Vec<u8> {
    let mut b = seed.to_vec();
    if b.is_empty() {
        b.reserve(EMPTY_SEED_BYTES);
        b.push(rng.byte());
    }
    b
}

pub(super) fn block_len(limit: usize, rng: &mut Rng) -> usize {
    if limit == 0 {
        0
    } else {
        1 + rng.below(HAVOC_BLK_SMALL.min(limit))
    }
}
