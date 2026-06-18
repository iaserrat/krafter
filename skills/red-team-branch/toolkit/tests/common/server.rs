use tiny_http::{Header, Response, Server};

struct Resp {
    code: u16,
    body: String,
    headers: Vec<(String, String)>,
}

pub fn start_server() -> u16 {
    let server = Server::http("127.0.0.1:0").expect("bind ephemeral");
    let port = server.server_addr().to_ip().expect("ip addr").port();
    std::thread::spawn(move || {
        for req in server.incoming_requests() {
            let r = handle(req.method().as_str(), req.url(), header_val(&req, "origin"), header_val(&req, "x-user"));
            let mut resp = Response::from_string(r.body).with_status_code(r.code);
            for (k, v) in &r.headers {
                resp.add_header(Header::from_bytes(k.as_bytes(), v.as_bytes()).unwrap());
            }
            let _ = req.respond(resp);
        }
    });
    wait_until_ready(port);
    port
}

fn wait_until_ready(port: u16) {
    for _ in 0..100 {
        if std::net::TcpStream::connect(("127.0.0.1", port)).is_ok() {
            break;
        }
        std::thread::yield_now();
    }
}

fn header_val(req: &tiny_http::Request, name: &str) -> Option<String> {
    req.headers()
        .iter()
        .find(|h| h.field.as_str().as_str().eq_ignore_ascii_case(name))
        .map(|h| h.value.as_str().to_string())
}

fn json(code: u16, body: &str) -> Resp {
    Resp { code, body: body.into(), headers: vec![ct("application/json")] }
}
fn ct(value: &str) -> (String, String) {
    ("Content-Type".into(), value.into())
}
fn handle(method: &str, url: &str, origin: Option<String>, user: Option<String>) -> Resp {
    let (path, query) = url.split_once('?').map_or((url, None), |(p, q)| (p, Some(q)));
    match path {
        p if p.starts_with("/records/") => records(method, p, user),
        "/echo" => echo(query),
        "/cors" => cors(origin),
        "/secure" => secure(),
        "/admin" => Resp { code: 200, body: "<html>admin panel</html>".into(), headers: vec![ct("text/html")] },
        "/.env" => Resp { code: 200, body: "SECRET=hunter2\nDB_PASS=p4ss".into(), headers: vec![ct("text/plain")] },
        _ => json(404, r#"{"error":"not found"}"#),
    }
}

fn records(method: &str, path: &str, user: Option<String>) -> Resp {
    if method == "DELETE" {
        return json(200, r#"{"deleted":true}"#);
    }
    if user.is_none() {
        return json(401, r#"{"error":"auth required"}"#);
    }
    let id = path.rsplit('/').next().unwrap_or("0");
    json(200, &format!(r#"{{"id":"{id}","owner_id":2,"ssn":"000-00-{id}","data":"rec {id}"}}"#))
}
fn echo(query: Option<&str>) -> Resp {
    for kv in query.unwrap_or("").split('&') {
        if let Some(value) = kv.strip_prefix("debug=") {
            return Resp { code: 200, body: format!("debug mode: {value}"), headers: vec![ct("text/plain")] };
        }
    }
    Resp { code: 200, body: "ok".into(), headers: vec![ct("text/plain")] }
}

// Broken CORS: reflects the caller's Origin and allows credentials.
fn cors(origin: Option<String>) -> Resp {
    let mut r = json(200, r#"{"ok":true}"#);
    r.headers.push(("Access-Control-Allow-Origin".into(), origin.unwrap_or_else(|| "*".into())));
    r.headers.push(("Access-Control-Allow-Credentials".into(), "true".into()));
    r
}

// All security headers present + one insecure cookie.
fn secure() -> Resp {
    let mut r = json(200, r#"{"ok":true}"#);
    let extra = [
        ("Strict-Transport-Security", "max-age=63072000"),
        ("Content-Security-Policy", "default-src 'self'"),
        ("X-Frame-Options", "DENY"),
        ("X-Content-Type-Options", "nosniff"),
        ("Set-Cookie", "sid=abc"),
    ];
    extra.iter().for_each(|(k, v)| r.headers.push(((*k).into(), (*v).into())));
    r
}
