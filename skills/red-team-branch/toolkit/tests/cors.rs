mod common;

use tiny_http::{Header, Response, Server};

// Safe own-origin allowlist: always answers with its OWN origin + credentials,
// never reflects the probe origin (evil/null/subdomain). Must be CLEARED.
fn start_safe_cors_server() -> u16 {
    let server = Server::http("127.0.0.1:0").expect("bind");
    let port = server.server_addr().to_ip().expect("ip").port();
    std::thread::spawn(move || {
        for req in server.incoming_requests() {
            let code = if req.method().as_str() == "OPTIONS" { 204 } else { 200 };
            let mut resp = Response::from_string("").with_status_code(code);
            for (k, v) in [
                ("Access-Control-Allow-Origin", "https://app.example.com"),
                ("Access-Control-Allow-Credentials", "true"),
                ("Access-Control-Allow-Methods", "GET, POST"),
            ] {
                resp.add_header(Header::from_bytes(k.as_bytes(), v.as_bytes()).unwrap());
            }
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

#[test]
fn cors_clears_safe_own_origin_allowlist() {
    let cfg = common::write_config(start_safe_cors_server());
    let v = common::run_rtk(&cfg, &["cors", "--url", "/", "--preflight"]);
    assert_eq!(v["verdict"], "CORS SAFE", "results: {:?}", v["results"]);
    assert_eq!(v["total_issues"], 0);
}

#[test]
fn cors_probe_runs_and_returns_json() {
    let port = common::start_server();
    let cfg = common::write_config(port);
    let v = common::run_rtk(&cfg, &[
        "cors", "--url", "/records/1",
        "--origin", "https://evil.com",
    ]);
    assert_eq!(v["probe"], "cors");
    assert!(v["origins_probed"].as_u64().unwrap() > 0);
}

#[test]
fn cors_accepts_custom_origins() {
    let port = common::start_server();
    let cfg = common::write_config(port);
    let v = common::run_rtk(&cfg, &[
        "cors", "--url", "/records/1",
        "--origin", "https://evil.com",
        "--origin", "null",
    ]);
    assert_eq!(v["probe"], "cors");
    assert!(v["origins_probed"].as_u64().unwrap() >= 2);
}

#[test]
fn cors_detects_reflected_origin_with_credentials() {
    let port = common::start_server();
    let cfg = common::write_config(port);
    let v = common::run_rtk(&cfg, &[
        "cors", "--url", "/cors", "--origin", "https://evil.com",
    ]);
    assert_eq!(v["verdict"], "CORS MISCONFIGURATION");
    let issues: Vec<&str> = v["results"][0]["issues"].as_array().unwrap()
        .iter().map(|i| i.as_str().unwrap()).collect();
    assert!(issues.contains(&"origin-reflected"), "issues: {issues:?}");
    assert!(issues.contains(&"credentials-allowed"), "issues: {issues:?}");
}