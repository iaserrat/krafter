use super::havoc::block_len;
use super::Rng;

const MIN_SPLICE_SOURCE_LEN: usize = 2;
const COPY_BLOCK_WEIGHT: usize = 4;
const COPY_BLOCK_MISS: usize = 0;

pub fn delete_block(b: &mut Vec<u8>, rng: &mut Rng) {
    let len = b.len();
    if len < MIN_SPLICE_SOURCE_LEN {
        return;
    }
    let del = block_len(len - 1, rng).max(1);
    let from = rng.below(len - del);
    b.drain(from..from + del);
}

pub fn insert_block(b: &mut Vec<u8>, rng: &mut Rng) {
    let len = b.len();
    let blk = block_len(len, rng).max(1);
    let at = rng.below(len + 1);
    let chunk = if rng.below(COPY_BLOCK_WEIGHT) != COPY_BLOCK_MISS {
        let from = rng.below(len - blk + 1);
        b[from..from + blk].to_vec()
    } else {
        vec![fill_byte(b, rng); blk]
    };
    splice_in(b, at, &chunk);
}

pub fn overwrite_block(b: &mut [u8], rng: &mut Rng) {
    let blk = block_len(b.len(), rng).max(1);
    let to = rng.below(b.len() - blk + 1);
    if rng.below(COPY_BLOCK_WEIGHT) != COPY_BLOCK_MISS {
        let from = rng.below(b.len() - blk + 1);
        let src = b[from..from + blk].to_vec();
        b[to..to + blk].copy_from_slice(&src);
    } else {
        let val = fill_byte(b, rng);
        for x in &mut b[to..to + blk] {
            *x = val;
        }
    }
}

fn splice_in(b: &mut Vec<u8>, at: usize, chunk: &[u8]) {
    let mut out = Vec::with_capacity(b.len() + chunk.len());
    out.extend_from_slice(&b[..at]);
    out.extend_from_slice(chunk);
    out.extend_from_slice(&b[at..]);
    *b = out;
}

fn fill_byte(b: &[u8], rng: &mut Rng) -> u8 {
    if rng.bool() {
        rng.byte()
    } else {
        b[rng.below(b.len())]
    }
}
