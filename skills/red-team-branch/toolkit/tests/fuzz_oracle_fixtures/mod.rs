//! Reflecting-vs-nonreflecting fixtures for the static-mode reflection oracle.
//! `/safe` returns a CONSTANT body that contains the payload substring but never
//! echoes the query. `/reflect` echoes the `q=` query value into the body.
use std::io::Cursor;
use tiny_http::{Header, Response, Server};

const CONSTANT_BODY: &str = "welcome to the admin dashboard, please sign in";

pub fn start_fixture() -> u16 {
    let server = Server::http("127.0.0.1:0").expect("bind ephemeral");
    let port = server.server_addr().to_ip().expect("ip addr").port();
    std::thread::spawn(move || {
        for req in server.incoming_requests() {
            let url = req.url().to_string();
            respond(req, &route(&url));
        }
    });
    wait_until_ready(port);
    port
}

fn route(url: &str) -> String {
    let path = url.split('?').next().unwrap_or(url);
    match path {
        // Constant body: contains "admin" substring but ignores input entirely.
        "/safe" => CONSTANT_BODY.to_string(),
        // Genuine reflection: echoes the raw q= value back into the body.
        "/reflect" => format!("results for: {}", query_value(url, "q")),
        _ => "not found".to_string(),
    }
}

fn query_value(url: &str, key: &str) -> String {
    url.split('?')
        .nth(1)
        .unwrap_or("")
        .split('&')
        .filter_map(|p| p.split_once('='))
        .find(|(k, _)| *k == key)
        .map(|(_, v)| v.to_string())
        .unwrap_or_default()
}

fn respond(req: tiny_http::Request, body: &str) {
    let ct = Header::from_bytes(&b"Content-Type"[..], &b"text/plain"[..]).unwrap();
    let resp = Response::new(200.into(), vec![ct], Cursor::new(body.as_bytes().to_vec()), None, None);
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
