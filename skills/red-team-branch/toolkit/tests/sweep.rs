mod common;
mod sweep_safe;

// NEGATIVE CONTROL: a server with correct per-user scoping (actor A reads only
// its own records, actor B cannot read A's, anon is denied) must yield NO
// cross_user_proven and a non-vulnerable verdict — the oracle must CLEAR it.
#[test]
fn sweep_clears_safe_per_user_scoped_target() {
    let port = sweep_safe::start_fixture();
    let cfg = common::write_config(port);
    let v = common::run_rtk(
        &cfg,
        &["sweep", "--url", "/records/{id}", "--range", "1-6", "--compare", "b"],
    );
    let proven = v["by_class"]["cross_user_proven"].as_u64().unwrap_or(0);
    assert_eq!(proven, 0, "safe target wrongly proven IDOR: {v:#}");
    let verdict = v["verdict"].as_str().unwrap();
    assert!(
        !verdict.contains("PROVEN") && !verdict.contains("LIKELY") && !verdict.contains("PUBLIC"),
        "safe target got vulnerable verdict {verdict:?}: {v:#}"
    );
}

#[test]
fn sweep_proves_cross_user_idor_and_surfaces_fields() {
    let port = common::start_server();
    let cfg = common::write_config(port);
    let v = common::run_rtk(
        &cfg,
        &[
            "sweep",
            "--url",
            "/records/{id}",
            "--range",
            "1-5",
            "--compare",
            "b",
        ],
    );
    assert!(v["verdict"].as_str().unwrap().contains("PROVEN IDOR"));
    let sensitive = v["sensitive_fields"].as_array().unwrap();
    assert!(
        sensitive.iter().any(|x| x == "ssn"),
        "expected ssn surfaced, got {sensitive:?}"
    );
}

#[test]
fn sweep_proves_idor_with_no_config_file_via_flags() {
    let port = common::start_server();
    let base = format!("http://127.0.0.1:{port}");
    let out = std::process::Command::new(env!("CARGO_BIN_EXE_rtk"))
        .current_dir(std::env::temp_dir())
        .args([
            "--base-url",
            &base,
            "--auth",
            "X-User: A",
            "--profile",
            "b: X-User: B",
            "sweep",
            "--url",
            "/records/{id}",
            "--range",
            "1-3",
            "--compare",
            "b",
        ])
        .output()
        .expect("run rtk");
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert!(v["verdict"].as_str().unwrap().contains("PROVEN IDOR"));
}
