const LOWER_CAP_BYTES: usize = 8192;

pub fn contains(haystack: &[u8], needle: &[u8]) -> bool {
    needle.is_empty()
        || (needle.len() <= haystack.len() && haystack.windows(needle.len()).any(|w| w == needle))
}

pub fn lower_capped(body: &[u8]) -> Vec<u8> {
    let mut v = body[..body.len().min(LOWER_CAP_BYTES)].to_vec();
    v.make_ascii_lowercase();
    v
}

pub fn blank(v: &mut [u8], pat: &[u8]) {
    if pat.is_empty() || pat.len() > v.len() {
        return;
    }
    let mut i = 0;
    while i + pat.len() <= v.len() {
        if &v[i..i + pat.len()] == pat {
            v[i..i + pat.len()].fill(b' ');
            i += pat.len();
        } else {
            i += 1;
        }
    }
}
