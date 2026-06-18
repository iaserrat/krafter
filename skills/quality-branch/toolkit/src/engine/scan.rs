//! Deterministic parallel map: source paths -> FileAnalysis, output order
//! independent of thread scheduling (sorted by path). Also the single guarded
//! source reader: skips oversized and binary files (git's NUL heuristic) so
//! generated/vendored blobs never pollute the metrics or crash the parser.

use crate::engine::{analyze, model::FileAnalysis};
use std::path::{Path, PathBuf};
use std::thread;

const PARALLEL_MIN_FILES: usize = 16;
const SERIAL_THREADS: usize = 1;
const PARALLEL_MAX_THREADS: usize = 16;
const MAX_FILE_BYTES: u64 = 1_048_576;
const BINARY_NUL: u8 = 0;
const BINARY_SNIFF_BYTES: usize = 8000;

/// A unit of work: repo-relative path (for language detection + reporting) plus
/// the absolute path to read.
pub struct Job {
    pub rel: String,
    pub abs: PathBuf,
}

/// Analyze every job; drops unparseable/oversized/binary files. Output is
/// sorted by path, so it is identical run-to-run regardless of scheduling.
pub fn analyze_all(jobs: Vec<Job>) -> Vec<FileAnalysis> {
    let workers = worker_count(jobs.len());
    let mut out = if workers <= SERIAL_THREADS {
        jobs.iter().filter_map(run_one).collect::<Vec<_>>()
    } else {
        run_parallel(&jobs, workers)
    };
    out.sort_by(|a, b| a.path.cmp(&b.path));
    out
}

fn run_parallel(jobs: &[Job], workers: usize) -> Vec<FileAnalysis> {
    let chunk = jobs.len().div_ceil(workers);
    thread::scope(|s| {
        let handles: Vec<_> = jobs
            .chunks(chunk)
            .map(|c| s.spawn(move || c.iter().filter_map(run_one).collect::<Vec<_>>()))
            .collect();
        handles.into_iter().flat_map(|h| h.join().unwrap()).collect()
    })
}

fn run_one(job: &Job) -> Option<FileAnalysis> {
    let bytes = read_source(&job.abs)?;
    let fa = analyze(Path::new(&job.rel), bytes);
    fa.parse_ok.then_some(fa)
}

/// Read a file unless oversized; then apply the in-memory guard. The single
/// guarded reader, shared by the repo scan and `metrics`.
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

fn worker_count(files: usize) -> usize {
    if files < PARALLEL_MIN_FILES {
        return SERIAL_THREADS;
    }
    thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(SERIAL_THREADS)
        .min(PARALLEL_MAX_THREADS)
        .min(files)
}
