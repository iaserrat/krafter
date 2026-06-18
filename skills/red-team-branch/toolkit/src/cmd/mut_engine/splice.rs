use super::Rng;

pub fn splice(a: &[u8], b: &[u8], rng: &mut Rng) -> Option<Vec<u8>> {
    if a.len() < 2 || b.len() < 2 {
        return None;
    }
    let n = a.len().min(b.len());
    let (mut first, mut last) = (None, None);
    for i in 0..n {
        if a[i] != b[i] {
            first.get_or_insert(i);
            last = Some(i);
        }
    }
    let (first, last) = (first?, last?);
    if last < 2 || first == last {
        return None;
    }
    let split = first + rng.below(last - first);
    let mut out = a[..split].to_vec();
    out.extend_from_slice(&b[split..]);
    Some(out)
}
