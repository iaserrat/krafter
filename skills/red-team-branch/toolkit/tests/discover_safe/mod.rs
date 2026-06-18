use tiny_http::{Header, Response, Server};

// Spawn a server whose handler maps (method,path) -> (code, body).
pub fn spawn(handler: fn(&str) -> (u16, String)) -> u16 {
    let server = Server::http("127.0.0.1:0").expect("bind");
    let port = server.server_addr().to_ip().expect("ip").port();
    std::thread::spawn(move || {
        for req in server.incoming_requests() {
            let (code, body) = handler(req.url());
            let mut resp = Response::from_string(body).with_status_code(code);
            let ct = Header::from_bytes(&b"Content-Type"[..], &b"text/html"[..]).unwrap();
            resp.add_header(ct);
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

// SPA catch-all: 200 for everything, body length varies a lot per path (echoes
// the path many times), so per-path bodies cross len-bucket boundaries.
pub fn spa_catch_all(url: &str) -> (u16, String) {
    (200, format!("<html><body>{}</body></html>", url.repeat(12)))
}

// Protected: unknown -> 404, /admin -> 401. Both bodies are the SAME length so
// the status-class + len-bucket signature collapses them identically.
pub fn protected(url: &str) -> (u16, String) {
    let path = url.split('?').next().unwrap_or(url);
    if path == "/admin" {
        (401, "AAAAAAAAAAAAAAAAAAAAAAAA".into())
    } else {
        (404, "BBBBBBBBBBBBBBBBBBBBBBBB".into())
    }
}
