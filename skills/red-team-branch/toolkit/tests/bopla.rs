mod common;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tiny_http::{Header, Method, Response, Server};

const ADMIN_FIELD: &str = "is_admin=true";
fn args(read: bool) -> Vec<&'static str> {
    let mut a = vec!["bopla", "--url", "/records/1"];
    if read {
        a.extend(["--read-url", "/records/1"]);
    }
    a.extend(["--field", ADMIN_FIELD]);
    a
}

// GET reads back is_admin reflecting committed state; the write echoes the body
// verbatim. `persist` commits on write; `preset` seeds the field before any write.
fn start_resource_server(persist: bool, preset: bool) -> u16 {
    let server = Server::http("127.0.0.1:0").expect("bind");
    let port = server.server_addr().to_ip().expect("ip").port();
    let committed = Arc::new(AtomicBool::new(preset));
    std::thread::spawn(move || {
        for mut req in server.incoming_requests() {
            let is_write = !matches!(req.method(), Method::Get);
            let body = if is_write {
                let mut buf = String::new();
                let _ = std::io::Read::read_to_string(req.as_reader(), &mut buf);
                if persist && buf.contains("\"is_admin\":true") {
                    committed.store(true, Ordering::SeqCst);
                }
                // Echo the submitted body verbatim (typical REST representation).
                buf
            } else if committed.load(Ordering::SeqCst) {
                r#"{"id":"1","is_admin":true}"#.into()
            } else {
                r#"{"id":"1","is_admin":false}"#.into()
            };
            let mut resp = Response::from_string(body).with_status_code(200);
            resp.add_header(
                Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap(),
            );
            let _ = req.respond(resp);
        }
    });
    for _ in 0..100 {
        if std::net::TcpStream::connect(("127.0.0.1", port)).is_ok() {
            break;
        }
        std::thread::yield_now();
    }
    port
}

// Negative control: value pre-exists (admin default), so before==after. MUST clear.
#[test]
fn bopla_clears_preexisting_value() {
    let cfg = common::write_config(start_resource_server(true, true));
    let v = common::run_rtk(&cfg, &args(true));
    assert_eq!(v["accepted_fields"].as_array().unwrap().len(), 0, "{v:#}");
}

// False-positive oracle: write echoes the body but independent read stays default. MUST clear.
#[test]
fn bopla_clears_echo_only_server() {
    let cfg = common::write_config(start_resource_server(false, false));
    let v = common::run_rtk(&cfg, &args(true));
    assert_eq!(v["accepted_fields"].as_array().unwrap().len(), 0, "{v:#}");
    assert!(
        v["verdict"].as_str().unwrap().to_lowercase().contains("not"),
        "echo-only must not be flagged: {v:#}"
    );
}

// Detection: server truly persists the field; independent read confirms. MUST flag.
#[test]
fn bopla_flags_persisting_server() {
    let cfg = common::write_config(start_resource_server(true, false));
    let v = common::run_rtk(&cfg, &args(true));
    let accepted: Vec<&str> = v["accepted_fields"]
        .as_array()
        .unwrap()
        .iter()
        .map(|f| f.as_str().unwrap())
        .collect();
    assert!(accepted.contains(&"is_admin"), "must flag persistence: {v:#}");
    assert!(
        v["verdict"].as_str().unwrap().contains("MASS ASSIGNMENT"),
        "{v:#}"
    );
}

// No --read-url: echo is indistinguishable from persistence. MUST be inconclusive.
#[test]
fn bopla_inconclusive_without_read_url() {
    let cfg = common::write_config(start_resource_server(true, false));
    let v = common::run_rtk(&cfg, &args(false));
    assert_eq!(v["accepted_fields"].as_array().unwrap().len(), 0, "{v:#}");
    assert_eq!(v["results"][0]["status"], "inconclusive", "{v:#}");
}
