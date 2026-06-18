use sha2::{Digest, Sha256};

const SHA8_BYTES: usize = 4;

pub fn sha8(bytes: &[u8]) -> String {
    let mut hash = Sha256::new();
    hash.update(bytes);
    hex::encode(&hash.finalize()[..SHA8_BYTES])
}
