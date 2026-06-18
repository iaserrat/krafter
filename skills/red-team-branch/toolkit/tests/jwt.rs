mod common;

#[test]
fn jwt_parses_and_generates_attacks() {
    let port = common::start_server();
    let cfg = common::write_config(port);
    let v = common::run_rtk(&cfg, &[
        "jwt",
        "--token", "eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIn0.Zm9vYmFy",
        "--verify-url", "/records/1",
    ]);
    assert_eq!(v["probe"], "jwt");
    assert!(v["attacks_probed"].as_u64().unwrap() > 0);
    let verdict = v["verdict"].as_str().unwrap();
    assert!(verdict.contains("JWT"));
}