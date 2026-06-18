mod common;
use common::{run_cqt, Repo};

#[test]
fn calibrate_reports_repo_distribution() {
    let repo = Repo::new();
    repo.write(
        "a.rs",
        "fn f1() {} fn f2(a: i32) -> i32 { if a > 0 { 1 } else { 0 } }",
    );
    repo.commit_all("init");

    let v = run_cqt(repo.path(), &["calibrate"]);

    assert_eq!(v["command"], "calibrate");
    assert!(v["functions"].as_u64().unwrap() >= 2);
    assert!(
        v["cognitive"]["max"].as_f64().unwrap() >= 1.0,
        "f2 has a branch, so max cognitive >= 1"
    );
}
