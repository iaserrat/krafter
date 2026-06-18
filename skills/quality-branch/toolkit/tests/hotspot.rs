mod common;
use common::{run_cqt, Repo};

#[test]
fn hotspot_ranks_churned_complex_file_first() {
    let repo = Repo::new();
    let hot = "fn h(a: i32) -> i32 { if a > 0 { if a > 1 { if a > 2 { a } else { 0 } } else { 0 } } else { 0 } }";

    // hot.rs: complex body, edited across four commits (high churn).
    for i in 0..4 {
        repo.write("hot.rs", &format!("{hot}\n// rev {i}"));
        repo.commit_all("edit hot");
    }
    // cold.rs: trivial, committed once.
    repo.write("cold.rs", "fn c() -> i32 { 1 }");
    repo.commit_all("add cold");

    let v = run_cqt(repo.path(), &["hotspot", "--top", "5"]);

    assert_eq!(v["command"], "hotspot");
    let hs = v["hotspots"].as_array().unwrap();
    assert_eq!(hs[0]["path"], "hot.rs", "churn x complexity should rank hot first");
    assert!(hs[0]["changes"].as_u64().unwrap() >= 4);
    assert!(hs[0]["score"].as_f64().unwrap() > 0.0);
}
