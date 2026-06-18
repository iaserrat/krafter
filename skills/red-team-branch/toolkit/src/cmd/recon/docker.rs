use serde_json::{json, Value};
use std::process::Command;

pub fn ps() -> Vec<Value> {
    let output = match Command::new("docker")
        .args(["ps", "--format", "{{json .}}"])
        .output()
    {
        Ok(out) if out.status.success() => out.stdout,
        Ok(_) => {
            eprintln!("[rtk][warn] `docker ps` returned non-zero (daemon down?)");
            return Vec::new();
        }
        Err(_) => {
            eprintln!("[rtk][warn] docker not found on PATH; skipping container enumeration");
            return Vec::new();
        }
    };
    String::from_utf8_lossy(&output)
        .lines()
        .filter_map(container)
        .collect()
}

fn container(line: &str) -> Option<Value> {
    let container = serde_json::from_str::<Value>(line).ok()?;
    let ports = container
        .get("Ports")
        .and_then(|p| p.as_str())
        .unwrap_or("");
    Some(json!({
        "name": container.get("Names").and_then(|v| v.as_str()),
        "image": container.get("Image").and_then(|v| v.as_str()),
        "status": container.get("Status").and_then(|v| v.as_str()),
        "ports_raw": ports,
        "host_ports": parse_host_ports(ports),
    }))
}

fn parse_host_ports(ports: &str) -> Vec<u16> {
    let mut out = ports
        .split(',')
        .filter_map(host_port_mapping)
        .collect::<Vec<_>>();
    out.sort_unstable();
    out.dedup();
    out
}

fn host_port_mapping(mapping: &str) -> Option<u16> {
    let (host_side, _) = mapping.split_once("->")?;
    host_side.rsplit(':').next()?.trim().parse().ok()
}
