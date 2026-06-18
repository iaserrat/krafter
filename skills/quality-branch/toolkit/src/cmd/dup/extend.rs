//! Extend a shared k-gram anchor maximally in both directions to recover the
//! full duplicated run. Pure and deterministic.

/// A maximal matching k-gram run, as inclusive k-gram indices in each file.
pub struct Run {
    pub a0: usize,
    pub a1: usize,
    pub b0: usize,
    pub b1: usize,
}

/// From a shared anchor (`ia` in a, `ib` in b), grow left then right while the
/// k-gram hashes keep matching.
pub fn extend(a: &[u64], ia: usize, b: &[u64], ib: usize) -> Run {
    let mut a0 = ia;
    let mut b0 = ib;
    while a0 > 0 && b0 > 0 && a[a0 - 1] == b[b0 - 1] {
        a0 -= 1;
        b0 -= 1;
    }
    let mut a1 = ia;
    let mut b1 = ib;
    while a1 + 1 < a.len() && b1 + 1 < b.len() && a[a1 + 1] == b[b1 + 1] {
        a1 += 1;
        b1 += 1;
    }
    Run { a0, a1, b0, b1 }
}
