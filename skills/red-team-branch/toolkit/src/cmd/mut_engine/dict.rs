use super::Rng;

pub fn dict_overwrite(b: &mut [u8], dict: &[Vec<u8>], rng: &mut Rng) {
    let d = &dict[rng.below(dict.len())];
    if !d.is_empty() && d.len() <= b.len() {
        let at = rng.below(b.len() - d.len() + 1);
        b[at..at + d.len()].copy_from_slice(d);
    }
}

pub fn dict_insert(b: &mut Vec<u8>, dict: &[Vec<u8>], rng: &mut Rng) {
    let d = &dict[rng.below(dict.len())];
    if d.is_empty() {
        return;
    }
    let at = rng.below(b.len() + 1);
    let mut out = Vec::with_capacity(b.len() + d.len());
    out.extend_from_slice(&b[..at]);
    out.extend_from_slice(d);
    out.extend_from_slice(&b[at..]);
    *b = out;
}
