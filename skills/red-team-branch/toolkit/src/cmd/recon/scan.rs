use futures::{stream, StreamExt};
use std::time::Duration;

pub async fn open_ports(
    host: &str,
    ports: &[u16],
    timeout_ms: u64,
    concurrency: usize,
) -> Vec<u16> {
    let timeout = Duration::from_millis(timeout_ms);
    let mut open = stream::iter(ports.iter().copied())
        .map(|port| probe(host.to_string(), port, timeout))
        .buffer_unordered(concurrency.max(32))
        .filter_map(|x| async move { x })
        .collect::<Vec<_>>()
        .await;
    open.sort_unstable();
    open
}

async fn probe(host: String, port: u16, timeout: Duration) -> Option<u16> {
    let addr = format!("{host}:{port}");
    let open = tokio::time::timeout(timeout, tokio::net::TcpStream::connect(addr)).await;
    matches!(open, Ok(Ok(_))).then_some(port)
}
