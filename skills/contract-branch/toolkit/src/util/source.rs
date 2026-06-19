//! The single guarded source reader: skips oversized and binary files (git's
//! NUL heuristic) so generated/vendored blobs never pollute a contract or crash
//! the parser. Shared by the `surface` command (working-tree reads) and the
//! base-side blob reads in `cmd`.

use std::path::Path;

const MAX_FILE_BYTES: u64 = 1_048_576;
const BINARY_NUL: u8 = 0;
const BINARY_SNIFF_BYTES: usize = 8000;

/// Read a file unless oversized; then apply the in-memory guard.
pub fn read_source(abs: &Path) -> Option<Vec<u8>> {
    let meta = std::fs::metadata(abs).ok()?;
    if meta.len() > MAX_FILE_BYTES {
        return None;
    }
    guard_bytes(std::fs::read(abs).ok()?)
}

/// Guard already-in-memory bytes (e.g. a git blob): reject oversized/binary.
pub fn guard_bytes(bytes: Vec<u8>) -> Option<Vec<u8>> {
    if bytes.len() as u64 > MAX_FILE_BYTES || is_binary(&bytes) {
        return None;
    }
    Some(bytes)
}

/// git's heuristic: a NUL in the first BINARY_SNIFF_BYTES means binary.
fn is_binary(bytes: &[u8]) -> bool {
    let n = bytes.len().min(BINARY_SNIFF_BYTES);
    bytes[..n].contains(&BINARY_NUL)
}
