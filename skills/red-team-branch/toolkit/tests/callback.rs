//! Drives the real rtk binary as an OOB listener: a call-home hit must be
//! captured to the log (path/query/body intact) and answered with respond_body.
use serde_json::Value;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::process::Command;

const CANARY_PATH: &str = "/ssrf-canary-001";
const CANARY_QUERY: &str = "leak=secret";
const CANARY_BODY: &str = "exfil=token-abc";
const RESPOND_BODY: &str = "rtk-callback ok";
const READY_POLLS: usize = 200;

fn free_port() -> u16 {
    TcpListener::bind("127.0.0.1:0").unwrap().local_addr().unwrap().port()
}

// Raw call-home: POST canary path+query+body; return the listener's response body.
fn call_home(port: u16) -> String {
    let mut s = TcpStream::connect(("127.0.0.1", port)).unwrap();
    let req = format!(
        "POST {CANARY_PATH}?{CANARY_QUERY} HTTP/1.1\r\nHost: victim.internal\r\n\
         Content-Length: {}\r\nConnection: close\r\n\r\n{CANARY_BODY}",
        CANARY_BODY.len()
    );
    s.write_all(req.as_bytes()).unwrap();
    let mut resp = String::new();
    s.read_to_string(&mut resp).unwrap();
    resp
}

fn wait_ready(port: u16) {
    for _ in 0..READY_POLLS {
        if TcpStream::connect(("127.0.0.1", port)).is_ok() {
            return;
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    panic!("callback listener never came up on {port}");
}

fn read_hit(log: &std::path::Path) -> Value {
    for _ in 0..READY_POLLS {
        if let Ok(s) = std::fs::read_to_string(log) {
            if let Some(line) = s.lines().next() {
                return serde_json::from_str(line).expect("hit line is JSON");
            }
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    panic!("no hit logged to {}", log.display());
}

#[test]
fn callback_captures_and_logs_a_hit() {
    let port = free_port();
    let log = std::env::temp_dir().join(format!("rtk_cb_{port}.jsonl"));
    let _ = std::fs::remove_file(&log);
    let mut child = Command::new(env!("CARGO_BIN_EXE_rtk"))
        .args(["callback", "--host", "127.0.0.1", "--port"])
        .arg(port.to_string())
        .arg("--log-file")
        .arg(&log)
        .spawn()
        .expect("spawn callback");

    wait_ready(port);
    let response = call_home(port);
    let hit = read_hit(&log);
    child.kill().ok();
    child.wait().ok();
    let _ = std::fs::remove_file(&log);

    // respond round-trip: listener answered our call-home with the configured body.
    assert!(response.contains(RESPOND_BODY), "no respond body: {response}");
    // capture fidelity: a regression that swaps/drops any field must fail here.
    assert_eq!(hit["tool"], "callback");
    assert_eq!(hit["method"], "POST");
    assert_eq!(hit["path"], CANARY_PATH);
    assert_eq!(hit["query"], CANARY_QUERY);
    assert_eq!(hit["host_header"], "victim.internal");
    assert_eq!(hit["body"], CANARY_BODY);
    assert_eq!(hit["body_len"], CANARY_BODY.len());
}
