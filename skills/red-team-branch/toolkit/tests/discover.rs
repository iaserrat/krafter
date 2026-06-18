mod common;
mod discover_safe;

fn route_paths(v: &serde_json::Value) -> Vec<String> {
    v["routes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r["path"].as_str().unwrap().to_string())
        .collect()
}

#[test]
fn discover_finds_sensitive_routes() {
    let port = common::start_server();
    let cfg = common::write_config(port);
    let v = common::run_rtk(&cfg, &["discover"]);
    let paths = route_paths(&v);
    assert!(paths.contains(&"admin".to_string()), "expected /admin, got {paths:?}");
    assert!(paths.contains(&".env".to_string()), "expected /.env, got {paths:?}");
}

// FALSE-POSITIVE ORACLE: a 200-everything SPA shell with per-path bodies must
// NOT be reported as a wall of live routes.
#[test]
fn discover_clears_spa_catch_all() {
    let cfg = common::write_config(discover_safe::spawn(discover_safe::spa_catch_all));
    let v = common::run_rtk(&cfg, &["discover"]);
    let paths = route_paths(&v);
    assert!(paths.len() <= 1, "spa catch-all flagged {} routes: {paths:?}", paths.len());
}

// FALSE-NEGATIVE ORACLE: /admin returning 401 (same status class + body length
// as the 404 baseline) must still be surfaced as a protected route.
#[test]
fn discover_surfaces_protected_admin() {
    let cfg = common::write_config(discover_safe::spawn(discover_safe::protected));
    let v = common::run_rtk(&cfg, &["discover"]);
    let paths = route_paths(&v);
    assert!(paths.contains(&"admin".to_string()), "expected protected /admin, got {paths:?}");
}
