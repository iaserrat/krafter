pub const DEFAULT_TIMEOUT_MS: u64 = 10_000;
pub const DEFAULT_CONCURRENCY: usize = 20;
pub const DEFAULT_USER_AGENT: &str = "rtk/0.1";

pub fn timeout_ms() -> u64 {
    DEFAULT_TIMEOUT_MS
}

pub fn concurrency() -> usize {
    DEFAULT_CONCURRENCY
}

pub fn user_agent() -> String {
    DEFAULT_USER_AGENT.into()
}
