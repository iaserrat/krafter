use crate::cmd::mut_engine::Rng;

const BOUNDARY_LEN: usize = 32;
const BOUNDARY_ALPHABET: &[u8] = b"0123456789abcdef";

pub fn random_boundary(rng: &mut Rng) -> String {
    let mut s = String::with_capacity(BOUNDARY_LEN);
    for _ in 0..BOUNDARY_LEN {
        s.push(BOUNDARY_ALPHABET[rng.below(BOUNDARY_ALPHABET.len())] as char);
    }
    s
}
