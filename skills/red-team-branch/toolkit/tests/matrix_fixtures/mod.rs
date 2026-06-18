//! Inline tiny_http fixtures for the matrix authz-gap oracle.
//! `/gated/*` is access-controlled (anon denied on GET) yet leaks mutations;
//! `/public/*` is public-by-design (everyone allowed) and must never be flagged.
use std::io::Cursor;
use tiny_http::{Header, Response, Server};

pub fn start_fixture() -> u16 {
    let server = Server::http("127.0.0.1:0").expect("bind ephemeral");
    let port = server.server_addr().to_ip().expect("ip addr").port();
    std::thread::spawn(move || {
        for req in server.incoming_requests() {
            let user = header_val(&req, "x-user");
            let (code, location) = route(req.method().as_str(), req.url(), user.is_some());
            respond(req, code, location);
        }
    });
    wait_until_ready(port);
    port
}

fn route(method: &str, url: &str, has_user: bool) -> (u16, Option<&'static str>) {
    let path = url.split('?').next().unwrap_or(url);
    match path {
        // Public by design: every verb succeeds for everyone, nobody is ever denied.
        p if p.starts_with("/public") => (200, None),
        // Access-controlled: GET gates on X-User, but mutations leak.
        p if p.starts_with("/gated") => gated(method, has_user),
        _ => (404, None),
    }
}

fn gated(method: &str, has_user: bool) -> (u16, Option<&'static str>) {
    match method {
        "GET" if !has_user => (401, None),
        "GET" => (200, None),
        "DELETE" => (200, None),         // authz gap: anon may delete
        "POST" => (303, Some("/gated")), // authz gap surfaced as a redirect (3xx)
        _ => (401, None),                // PUT/PATCH are correctly gated
    }
}

fn respond(req: tiny_http::Request, code: u16, location: Option<&str>) {
    let mut resp = Response::new(code.into(), vec![], Cursor::new(Vec::new()), Some(0), None);
    if let Some(loc) = location {
        resp.add_header(Header::from_bytes(&b"Location"[..], loc.as_bytes()).unwrap());
    }
    let _ = req.respond(resp);
}

fn header_val(req: &tiny_http::Request, name: &str) -> Option<String> {
    req.headers()
        .iter()
        .find(|h| h.field.as_str().as_str().eq_ignore_ascii_case(name))
        .map(|h| h.value.as_str().to_string())
}

fn wait_until_ready(port: u16) {
    for _ in 0..100 {
        if std::net::TcpStream::connect(("127.0.0.1", port)).is_ok() {
            break;
        }
        std::thread::yield_now();
    }
}
