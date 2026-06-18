use super::Args;
use crate::util;
use serde_json::Value;

pub fn collect(args: &Args, containers: &[Value]) -> Vec<u16> {
    let mut ports = args
        .ports
        .as_deref()
        .map(util::parse_ports)
        .unwrap_or_else(util::default_ports);
    for container in containers {
        if let Some(host_ports) = container.get("host_ports").and_then(|p| p.as_array()) {
            ports.extend(
                host_ports
                    .iter()
                    .filter_map(|p| p.as_u64().map(|n| n as u16)),
            );
        }
    }
    ports.sort_unstable();
    ports.dedup();
    ports
}
