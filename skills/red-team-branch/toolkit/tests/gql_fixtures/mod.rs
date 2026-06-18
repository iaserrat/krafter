//! Inline tiny_http fixtures for the gql batch oracle's negative control.
//! `/batch-errors` returns a 2-element array where EVERY element is an error
//! with no executed `data` (batch rejected). `/vuln` is a real GraphQL server:
//! introspection returns schema types and a batch returns per-element `data`.
use std::io::Cursor;
use tiny_http::{Header, Response, Server};

pub fn start_fixture() -> u16 {
    let server = Server::http("127.0.0.1:0").expect("bind ephemeral");
    let port = server.server_addr().to_ip().expect("ip addr").port();
    std::thread::spawn(move || {
        for mut req in server.incoming_requests() {
            let path = req.url().split('?').next().unwrap_or("/").to_string();
            let mut body = String::new();
            let _ = req.as_reader().read_to_string(&mut body);
            respond(req, &route(&path, &body));
        }
    });
    wait_until_ready(port);
    port
}

// A request body starting with '[' is a batch; otherwise a single operation.
fn route(path: &str, body: &str) -> String {
    let is_batch = body.trim_start().starts_with('[');
    match path {
        "/batch-errors" => batch_errors().into(),
        "/vuln" if is_batch => vuln_batch().into(),
        "/vuln" => vuln_single().into(),
        _ => r#"{"errors":[{"message":"not found"}]}"#.into(),
    }
}

// Two-element array, every element a pure error with null/absent data.
fn batch_errors() -> &'static str {
    r#"[{"data":null,"errors":[{"message":"x"}]},{"data":null,"errors":[{"message":"y"}]}]"#
}

fn vuln_batch() -> &'static str {
    r#"[{"data":{"__typename":"Query"}},{"data":{"__schema":{"types":[{"name":"Query"}]}}}]"#
}

fn vuln_single() -> &'static str {
    r#"{"data":{"__schema":{"types":[{"name":"Query"},{"name":"User"}]}}}"#
}

fn respond(req: tiny_http::Request, body: &str) {
    let ct = Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap();
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
