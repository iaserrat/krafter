mod common;
mod recon_fixtures;

use serde_json::Value;

// recon drives --host/--ports, not config base_url; config is only needed so
// run_rtk has a file. Point recon at the exact fixture ports.
fn recon(ports: &[u16]) -> Value {
    let cfg = common::write_config(ports[0]);
    let list = ports.iter().map(u16::to_string).collect::<Vec<_>>().join(",");
    common::run_rtk(&cfg, &["recon", "--ports", &list])
}

fn open_ports(v: &Value) -> Vec<u64> {
    v["open_ports"].as_array().unwrap().iter().map(|p| p.as_u64().unwrap()).collect()
}

fn service_ports(v: &Value) -> Vec<u64> {
    v["http_services"].as_array().unwrap().iter().map(|s| s["port"].as_u64().unwrap()).collect()
}

// DETECTION: a live HTTP service must surface in http_services with its status,
// server banner and parsed title.
#[test]
fn recon_reports_http_service() {
    let port = recon_fixtures::http_service();
    let v = recon(&[port]);
    let svc = v["http_services"].as_array().unwrap();
    let hit = svc.iter().find(|s| s["port"].as_u64() == Some(port as u64));
    let hit = hit.unwrap_or_else(|| panic!("http service port {port} missing: {svc:?}"));
    assert_eq!(hit["status"].as_u64(), Some(200), "status: {hit:?}");
    assert_eq!(hit["server"].as_str(), Some("nginx/1.25"), "server: {hit:?}");
    assert_eq!(hit["title"].as_str(), Some("Live App"), "title: {hit:?}");
}

// FALSE-POSITIVE ORACLE: an open port speaking no HTTP must appear in open_ports
// but must NOT be mislabeled as an http_service.
#[test]
fn recon_lists_raw_tcp_open_but_not_http() {
    let port = recon_fixtures::raw_tcp();
    let v = recon(&[port]);
    assert!(open_ports(&v).contains(&(port as u64)), "open_ports missing {port}: {v}");
    assert!(!service_ports(&v).contains(&(port as u64)), "raw TCP mislabeled http: {v}");
}

// ROOT-CAUSE ORACLE: a service whose body read fails (headers received, body
// truncated mid-stream) must still be reported with status/server and an empty
// title, not dropped from http_services.
#[test]
fn recon_keeps_service_when_body_read_fails() {
    let port = recon_fixtures::truncated_body();
    let v = recon(&[port]);
    let svc = v["http_services"].as_array().unwrap();
    let hit = svc.iter().find(|s| s["port"].as_u64() == Some(port as u64));
    let hit = hit.unwrap_or_else(|| panic!("stalled service port {port} dropped: {svc:?}"));
    assert_eq!(hit["status"].as_u64(), Some(200), "status lost: {hit:?}");
    assert_eq!(hit["server"].as_str(), Some("stall/1.0"), "server lost: {hit:?}");
    assert_eq!(hit["title"].as_str(), None, "expected empty title: {hit:?}");
}
