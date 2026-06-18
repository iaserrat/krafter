//! Inline tiny_http fixtures for the params status-class negative control.
//! `/flaky` alternates 200/503 with a CONSTANT body (no param reacts) so a
//! status-class oracle without a noisy guard mints false findings.
//! `/reflect` flakes status too but echoes the canary when `debug=` is present,
//! proving a genuinely reacting param is still detected on a noisy endpoint.
use std::io::Cursor;
use std::sync::atomic::{AtomicU32, Ordering};
use tiny_http::{Header, Response, Server};

const FLAKY_MODULUS: u32 = 2;
const OK: u16 = 200;
const UNAVAILABLE: u16 = 503;

pub fn start_fixture() -> u16 {
    let server = Server::http("127.0.0.1:0").expect("bind ephemeral");
    let port = server.server_addr().to_ip().expect("ip addr").port();
    std::thread::spawn(move || {
        let counter = AtomicU32::new(0);
        for req in server.incoming_requests() {
            let n = counter.fetch_add(1, Ordering::SeqCst);
            let (code, body) = route(req.url(), n);
            respond(req, code, body);
        }
    });
    wait_until_ready(port);
    port
}

// Status alternates per request; body length is constant so the body_fp/len
// noisy detector alone would miss the flakiness. Only `/reflect` echoes input.
fn route(url: &str, n: u32) -> (u16, String) {
    let code = if n % FLAKY_MODULUS == 0 { OK } else { UNAVAILABLE };
    let path = url.split('?').next().unwrap_or(url);
    match path {
        "/reflect" => (code, reflect_body(url)),
        _ => (code, "constant body padding xxxxxxxxxx".into()),
    }
}

// Echo the injected `debug=` value so the canary reflects (constant length pad
// keeps len-bucket stable for non-reacting params).
fn reflect_body(url: &str) -> String {
    let echoed = url
        .split('?')
        .nth(1)
        .unwrap_or("")
        .split('&')
        .find_map(|kv| kv.strip_prefix("debug="))
        .unwrap_or("none");
    format!("constant body echo={echoed} xxxxxxxxxx")
}

fn respond(req: tiny_http::Request, code: u16, body: String) {
    let ct = Header::from_bytes(&b"Content-Type"[..], &b"text/plain"[..]).unwrap();
    let resp = Response::new(code.into(), vec![ct], Cursor::new(body.into_bytes()), None, None);
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
