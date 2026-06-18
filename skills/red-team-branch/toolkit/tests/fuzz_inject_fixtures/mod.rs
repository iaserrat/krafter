//! Reflecting fixtures for the fuzz mutate-mode injector channels.
//! Every route echoes its channel input back into the body so the reflection
//! oracle fires; that lets a test assert detection AND a faithful repro.
use std::io::{Cursor, Read};
use tiny_http::{Header, Request, Response, Server};

pub fn start_fixture() -> u16 {
    let server = Server::http("127.0.0.1:0").expect("bind ephemeral");
    let port = server.server_addr().to_ip().expect("ip addr").port();
    std::thread::spawn(move || {
        for req in server.incoming_requests() {
            respond(req);
        }
    });
    wait_until_ready(port);
    port
}

// `/reflect?q=` echoes the URL query (url channel); `/header` echoes the
// X-Probe header (header channel); `/echo` echoes the raw request body
// (body and multipart channels — multipart's framed part lands here verbatim).
fn respond(mut req: Request) {
    let url = req.url().to_string();
    let probe = header_val(&req, "x-probe").unwrap_or_default();
    let mut body = Vec::new();
    let _ = req.as_reader().read_to_end(&mut body);
    let path = url.split('?').next().unwrap_or(&url);
    let echoed = match path {
        "/reflect" => format!("query was: {}", query_value(&url, "q")),
        "/header" => format!("header was: {probe}"),
        _ => String::from_utf8_lossy(&body).into_owned(),
    };
    reply(req, &echoed);
}

fn header_val(req: &Request, name: &str) -> Option<String> {
    req.headers()
        .iter()
        .find(|h| h.field.as_str().as_str().eq_ignore_ascii_case(name))
        .map(|h| h.value.as_str().to_string())
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

fn reply(req: Request, body: &str) {
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
