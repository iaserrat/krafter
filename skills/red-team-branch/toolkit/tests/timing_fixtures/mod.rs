//! Inline tiny_http fixtures for the timing oracle, driven via the `{VAR}` body.
//! `/login` is constant-time: identical sub-ms work for ANY value (must CLEAR).
//! `/enum` leaks: it does extra sub-ms work ONLY for the valid value, a
//! microsecond-scale delta that ms-resolution truncates to 0 (must FLAG).
use std::hint::black_box;
use std::io::{Cursor, Read};
use tiny_http::{Header, Request, Response, Server};

const VALID: &str = "alice";
// Busy-loop iteration counts tuned to ~hundreds of microseconds, well under 1ms.
const BASE_WORK: u64 = 60_000;
const LEAK_WORK: u64 = 240_000;

pub fn start_fixture() -> u16 {
    let server = Server::http("127.0.0.1:0").expect("bind ephemeral");
    let port = server.server_addr().to_ip().expect("ip addr").port();
    std::thread::spawn(move || {
        for mut req in server.incoming_requests() {
            let path = req.url().split('?').next().unwrap_or("/").to_string();
            let mut body = String::new();
            let _ = req.as_reader().read_to_string(&mut body);
            spin(work_for(&path, &body));
            respond(req);
        }
    });
    wait_until_ready(port);
    port
}

// `/enum` spends extra cycles only when the submitted value is valid (a non-
// constant-time compare leak); `/login` spends the same cycles for every value.
fn work_for(path: &str, body: &str) -> u64 {
    let is_valid = body.contains(VALID);
    match path {
        "/enum" if is_valid => BASE_WORK + LEAK_WORK,
        _ => BASE_WORK,
    }
}

// Deterministic CPU work; black_box prevents the optimizer from eliding it.
fn spin(iters: u64) {
    let mut acc: u64 = 0;
    for i in 0..iters {
        acc = acc.wrapping_add(black_box(i).wrapping_mul(2_654_435_761));
    }
    black_box(acc);
}

fn respond(req: Request) {
    let ct = Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap();
    let body = br#"{"ok":true}"#.to_vec();
    let resp = Response::new(200.into(), vec![ct], Cursor::new(body), None, None);
    let _ = req.respond(resp);
}

fn wait_until_ready(port: u16) {
    for _ in 0..100 {
        if std::net::TcpStream::connect(("127.0.0.1", port)).is_ok() {
            break;
        }
        std::thread::yield_now();
    }
}
