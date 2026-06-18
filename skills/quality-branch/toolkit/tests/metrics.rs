mod common;
use common::{run_cqt, Repo};

#[test]
fn metrics_reports_functions_with_cognitive() {
    let repo = Repo::new();
    repo.write(
        "calc.rs",
        "fn rate(a: i32) -> i32 { if a > 0 { if a > 1 { a } else { 0 } } else { 0 } }",
    );
    repo.commit_all("init");

    let abs = repo.path().join("calc.rs");
    let v = run_cqt(repo.path(), &["metrics", "--paths", abs.to_str().unwrap()]);

    assert_eq!(v["command"], "metrics");
    assert_eq!(v["parse_errors"], 0);
    let fns = v["files"][0]["functions"].as_array().unwrap();
    let rate = fns
        .iter()
        .find(|f| f["name"].as_str().unwrap().contains("rate"))
        .expect("rate function measured");
    assert!(rate["cognitive"].as_f64().unwrap() > 0.0);
}

#[test]
fn metrics_flags_unsupported_files() {
    let repo = Repo::new();
    repo.write("notes.bin", "not source");
    repo.commit_all("init");

    let abs = repo.path().join("notes.bin");
    let v = run_cqt(repo.path(), &["metrics", "--paths", abs.to_str().unwrap()]);

    assert_eq!(v["parse_errors"], 1);
    assert_eq!(v["files"][0]["parse_ok"], false);
}
