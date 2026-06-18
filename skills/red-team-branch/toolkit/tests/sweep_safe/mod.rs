//! SAFE per-user-scoped fixture for the sweep IDOR oracle's negative control.
//! Each record is owned by exactly one user; an actor reads only its own,
//! every other id is 403, and anon (no X-User) is always denied.
use tiny_http::{Header, Response, Server};

pub fn start_fixture() -> u16 {
    let server = Server::http("127.0.0.1:0").expect("bind ephemeral");
    let port = server.server_addr().to_ip().expect("ip addr").port();
    std::thread::spawn(move || {
        for req in server.incoming_requests() {
            let user = header_val(&req, "x-user");
            let r = record(req.url(), user.as_deref());
            let mut resp = Response::from_string(r.body).with_status_code(r.code);
            let ct = Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap();
            resp.add_header(ct);
            let _ = req.respond(resp);
        }
    });
    wait_until_ready(port);
    port
}

struct Resp {
    code: u16,
    body: String,
}

// Owner of record id N is the user whose name hashes to N's parity:
// odd ids -> "A", even ids -> "B". Only the owner may read; nobody else can.
fn record(url: &str, user: Option<&str>) -> Resp {
    let id = url.rsplit('/').next().unwrap_or("0");
    let user = match user {
        Some(u) => u,
        None => return Resp { code: 401, body: r#"{"error":"auth required"}"#.into() },
    };
    let owner = owner_of(id);
    if user != owner {
        return Resp { code: 403, body: r#"{"error":"forbidden"}"#.into() };
    }
    Resp {
        code: 200,
        body: format!(r#"{{"id":"{id}","owner":"{owner}","data":"private {owner} {id}"}}"#),
    }
}

fn owner_of(id: &str) -> &'static str {
    match id.bytes().next_back() {
        Some(b) if (b - b'0') % 2 == 1 => "A",
        _ => "B",
    }
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
