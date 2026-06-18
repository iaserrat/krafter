mod common;

use serde_json::Value;
use std::process::Command;

// Constant-response server: every request returns the same body, so the
// fuzzer discovers no new coverage and the corpus stays at the seed count.
fn start_static_server() -> u16 {
    let server = tiny_http::Server::http("127.0.0.1:0").expect("bind ephemeral");
    let port = server.server_addr().to_ip().expect("ip addr").port();
    std::thread::spawn(move || {
        for req in server.incoming_requests() {
            let resp = tiny_http::Response::from_string("ok").with_status_code(200);
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

fn run_fuzz(cfg: &str, state: &str, seed: &str) -> Value {
    let out = Command::new(env!("CARGO_BIN_EXE_rtk"))
        .arg("--config")
        .arg(cfg)
        .args([
            "fuzz", "--mutate", "--url", "/records/{FUZZ}", "--seed", seed, "--max-exec", "60",
            "--state", state,
        ])
        .output()
        .expect("run rtk");
    serde_json::from_slice(&out.stdout).unwrap_or_else(|e| {
        panic!(
            "rtk produced non-JSON ({e})\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        )
    })
}

// Resume must dedup: re-running with the same seed against a server that yields
// no new coverage must NOT re-inject the saved seeds, so the corpus is stable.
#[test]
fn resume_does_not_accumulate_duplicate_seeds() {
    let port = start_static_server();
    let cfg = common::write_config(port);
    let state = std::env::temp_dir().join(format!("rtk_ff_{port}.json"));
    let _ = std::fs::remove_file(&state);
    let st = state.to_str().unwrap();

    let c1 = run_fuzz(&cfg, st, "7")["stats"]["corpus_size"]
        .as_u64()
        .unwrap();
    let c2 = run_fuzz(&cfg, st, "7")["stats"]["corpus_size"]
        .as_u64()
        .unwrap();

    let _ = std::fs::remove_file(&state);
    assert_eq!(
        c1, c2,
        "resume re-injected the saved corpus: run1={c1} run2={c2}"
    );
}
