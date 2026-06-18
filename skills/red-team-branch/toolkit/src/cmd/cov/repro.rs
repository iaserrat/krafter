#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct Repro {
    pub payload: Vec<u8>,
    pub minimized: Vec<u8>,
    pub oracle_mask: u8,
    pub family: u8,
    pub status: u16,
    pub body_len: usize,
    pub latency_ms: u64,
    pub iter: u64,
    pub snippet: String,
}
