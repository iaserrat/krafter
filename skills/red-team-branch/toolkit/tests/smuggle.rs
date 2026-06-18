mod common;

use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

// Raw single-origin fixture: never desyncs, never reflects the canary, but the
// follow-up GET / lands on a benign 503 (overload) while the baseline saw 200.
// A correct probe must NOT escalate a flaky status flip to a smuggling verdict.
fn start_flaky_status_server() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
    let port = listener.local_addr().unwrap().port();
    let gets = Arc::new(AtomicUsize::new(0));
    std::thread::spawn(move || {
        for stream in listener.incoming().flatten() {
            serve_flaky(stream, &gets);
        }
    });
    port
}

fn serve_flaky(mut stream: std::net::TcpStream, gets: &Arc<AtomicUsize>) {
    let mut buf = [0u8; 4096];
    let n = stream.read(&mut buf).unwrap_or(0);
    let line = String::from_utf8_lossy(&buf[..n]);
    // First GET is the baseline (200); every later GET is a benign overload (503).
    let code = if line.starts_with("GET") {
        if gets.fetch_add(1, Ordering::SeqCst) == 0 { 200 } else { 503 }
    } else {
        400 // malformed smuggle payload, rejected
    };
    let resp = format!("HTTP/1.1 {code} X\r\nContent-Length: 2\r\nConnection: close\r\n\r\nok");
    let _ = stream.write_all(resp.as_bytes());
}

#[test]
fn smuggle_does_not_cry_wolf_on_flaky_status() {
    let port = start_flaky_status_server();
    let cfg = common::write_config(port);
    let url = format!("http://127.0.0.1:{port}/");
    let v = common::run_rtk(&cfg, &["smuggle", "--url", url.as_str()]);
    assert_eq!(v["probe"], "smuggle");
    assert_eq!(v["techniques_probed"], 3);
    // Benign status flicker between baseline and follow-up is NOT a desync.
    assert_eq!(
        v["verdict"], "NO SMUGGLING DETECTED",
        "flaky status must not be flagged; results: {:?}", v["results"]
    );
    assert_eq!(v["exploitable_count"], 0);
}

#[test]
fn smuggle_probes_all_techniques_without_false_positive() {
    let port = common::start_server();
    let cfg = common::write_config(port);
    let url = format!("http://127.0.0.1:{port}/");
    let v = common::run_rtk(&cfg, &["smuggle", "--url", url.as_str()]);
    assert_eq!(v["probe"], "smuggle");
    assert_eq!(v["techniques_probed"], 3);
    // A single origin server cannot desync; the probe must not cry wolf.
    assert_eq!(v["verdict"], "NO SMUGGLING DETECTED");
}
