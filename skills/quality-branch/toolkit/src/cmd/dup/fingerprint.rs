//! FNV-1a hashing and k-gram construction. Fixed, dependency-free, fully
//! deterministic — no rolling state, no RNG.

const FNV_OFFSET: u64 = 0xcbf29ce484222325;
const FNV_PRIME: u64 = 0x100000001b3;

/// FNV-1a over raw bytes (used to hash one canonical token).
pub fn fnv1a(bytes: &[u8]) -> u64 {
    let mut h = FNV_OFFSET;
    for &b in bytes {
        h ^= b as u64;
        h = h.wrapping_mul(FNV_PRIME);
    }
    h
}

/// FNV-1a over a slice of token hashes (the k-gram hash: hash-of-hashes).
fn hash_u64s(hs: &[u64]) -> u64 {
    let mut h = FNV_OFFSET;
    for &x in hs {
        for b in x.to_le_bytes() {
            h ^= b as u64;
            h = h.wrapping_mul(FNV_PRIME);
        }
    }
    h
}

/// One hash per contiguous k-token window. Empty if k is 0 or fewer than k
/// tokens (k=0 is undefined and would fabricate a window for an empty file).
pub fn kgrams(token_hashes: &[u64], k: usize) -> Vec<u64> {
    if k == 0 || token_hashes.len() < k {
        return Vec::new();
    }
    (0..=token_hashes.len() - k)
        .map(|i| hash_u64s(&token_hashes[i..i + k]))
        .collect()
}
