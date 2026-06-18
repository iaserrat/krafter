//! Inline tiny_http fixtures for the race oracle's negative control.
//! `/locked` is idempotent: every concurrent winner gets the SAME 200 body
//! (one real effect). `/nolock` double-spends: each winner gets a DISTINCT body.
use std::io::Cursor;
use std::sync::atomic::{AtomicU32, Ordering};
use tiny_http::{Response, Server};

pub fn start_fixture() -> u16 {
    let server = Server::http("127.0.0.1:0").expect("bind ephemeral");
    let port = server.server_addr().to_ip().expect("ip addr").port();
    std::thread::spawn(move || {
        let counter = AtomicU32::new(0);
        for req in server.incoming_requests() {
            let body = route(req.url(), &counter);
            respond(req, body);
        }
    });
    wait_until_ready(port);
    port
}

// Both paths answer 200 to every attempt, so a success-count oracle trips both.
// The effect (body) is what distinguishes a locked endpoint from a double-spend.
fn route(url: &str, counter: &AtomicU32) -> String {
    let path = url.split('?').next().unwrap_or(url);
    match path {
        "/locked" => r#"{"redeemed":true,"order":"ORD-1"}"#.into(),
        "/nolock" => {
            let n = counter.fetch_add(1, Ordering::SeqCst);
            format!(r#"{{"redeemed":true,"order":"ORD-{n}"}}"#)
        }
        _ => r#"{"error":"not found"}"#.into(),
    }
}

fn respond(req: tiny_http::Request, body: String) {
    let resp = Response::new(
        200.into(),
        vec![],
        Cursor::new(body.into_bytes()),
        None,
        None,
    );
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
