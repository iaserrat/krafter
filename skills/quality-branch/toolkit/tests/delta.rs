mod common;
use common::{run_cqt, Repo};

#[test]
fn delta_flags_increased_cognitive_complexity() {
    let repo = Repo::new();
    repo.write("calc.rs", "fn rate(a: i32) -> i32 { a }");
    repo.commit_all("base: simple");

    repo.checkout_new("feature");
    repo.write(
        "calc.rs",
        "fn rate(a: i32) -> i32 { if a > 0 { if a > 1 { for _ in 0..a { return a; } } } 0 }",
    );
    repo.commit_all("feature: nested");

    let v = run_cqt(repo.path(), &["--base", "main", "delta"]);

    assert_eq!(v["command"], "delta");
    assert_eq!(v["changed_files"], 1);
    let fns = v["functions"].as_array().unwrap();
    let rate = fns
        .iter()
        .find(|f| f["name"].as_str().unwrap().contains("rate"))
        .expect("rate appears in the delta");
    assert_eq!(rate["status"], "changed");
    assert!(
        rate["delta_cognitive"].as_f64().unwrap() > 0.0,
        "nesting should raise cognitive complexity"
    );
}

#[test]
fn delta_follows_renames_not_add_remove() {
    let repo = Repo::new();
    let body = "fn rate(a: i32) -> i32 { let mut s = 0; for v in 0..a { if v > 0 { s += v; } } s }";
    repo.write("old.rs", body);
    repo.commit_all("base");

    repo.checkout_new("feature");
    repo.remove("old.rs");
    // same function, one extra branch -> high similarity, git detects the rename.
    repo.write(
        "new.rs",
        "fn rate(a: i32) -> i32 { let mut s = 0; for v in 0..a { if v > 0 { if v > 1 { s += v; } } } s }",
    );
    repo.commit_all("rename + tweak");

    let v = run_cqt(repo.path(), &["--base", "main", "delta"]);
    let fns = v["functions"].as_array().unwrap();
    // rate is reported once, under the NEW path, as changed — never add+remove.
    let rate: Vec<_> = fns
        .iter()
        .filter(|f| f["name"].as_str().unwrap().contains("rate"))
        .collect();
    assert_eq!(rate.len(), 1, "rename must not split into add+remove");
    assert_eq!(rate[0]["status"], "changed");
    assert_eq!(rate[0]["path"], "new.rs");
}

#[test]
fn delta_reports_added_function() {
    let repo = Repo::new();
    repo.write("m.rs", "fn a() {}");
    repo.commit_all("base");

    repo.checkout_new("feature");
    repo.write("m.rs", "fn a() {} fn b(x: i32) -> i32 { if x > 0 { 1 } else { 0 } }");
    repo.commit_all("add b");

    let v = run_cqt(repo.path(), &["--base", "main", "delta"]);
    let fns = v["functions"].as_array().unwrap();
    let b = fns
        .iter()
        .find(|f| f["name"].as_str().unwrap().contains("b"))
        .expect("new function b appears");
    assert_eq!(b["status"], "added");
    assert_eq!(b["before_cognitive"].as_f64().unwrap(), 0.0);
}
