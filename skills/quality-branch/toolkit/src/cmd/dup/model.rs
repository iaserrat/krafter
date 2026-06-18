use serde::Serialize;

/// One duplicated span. 1-based inclusive lines, repo-relative path.
#[derive(Serialize)]
pub struct Member {
    pub path: String,
    pub start_line: usize,
    pub end_line: usize,
}

/// A set of duplicated spans (>=2). `token_length` is the canonical clone size.
#[derive(Serialize)]
pub struct CloneGroup {
    pub token_length: usize,
    pub line_length: usize,
    pub members: Vec<Member>,
}
