mod common;
use common::{run_cqt, Repo};

/// Build history where a.rs and b.rs always change together, c.rs alone.
fn co_change_repo() -> Repo {
    let repo = Repo::new();
    repo.write("a.rs", "fn a() {}");
    repo.write("b.rs", "fn b() {}");
    repo.commit_all("c1: a+b");
    repo.write("a.rs", "fn a() { let x = 1; }");
    repo.write("b.rs", "fn b() { let y = 1; }");
    repo.commit_all("c2: a+b");
    repo.write("a.rs", "fn a() { let x = 2; }");
    repo.write("b.rs", "fn b() { let y = 2; }");
    repo.commit_all("c3: a+b");
    repo.write("c.rs", "fn c() {}");
    repo.commit_all("c4: c alone");
    repo
}

#[test]
fn coupling_flags_missing_co_change() {
    let repo = co_change_repo();
    repo.checkout_new("feature");
    repo.write("a.rs", "fn a() { let x = 3; }"); // touch a, NOT b
    repo.commit_all("edit a only");

    let v = run_cqt(
        repo.path(),
        &["--base", "main", "coupling", "--min-shared", "2", "--min-revs", "2", "--min-degree", "0.3"],
    );
    assert_eq!(v["command"], "coupling");
    assert_eq!(v["scope"], "branch");
    let pairs = v["pairs"].as_array().unwrap();
    let ab = pairs
        .iter()
        .find(|p| p["file_a"] == "a.rs" && p["file_b"] == "b.rs")
        .expect("a.rs/b.rs coupling surfaced");
    assert_eq!(ab["status"], "missed", "b.rs was not touched but usually co-changes");
    assert_eq!(ab["anchor"], "a.rs");
    assert!(ab["degree"].as_f64().unwrap() >= 0.99);
}

#[test]
fn coupling_global_scope_is_unannotated_and_deterministic() {
    let repo = co_change_repo();
    let args = ["coupling", "--scope", "global", "--min-shared", "2", "--min-revs", "2", "--min-degree", "0.3"];
    let v = run_cqt(repo.path(), &args);
    let pairs = v["pairs"].as_array().unwrap();
    assert!(pairs.iter().any(|p| p["file_a"] == "a.rs" && p["file_b"] == "b.rs"));
    // global scope carries no branch annotation
    assert!(pairs[0].get("status").is_none());
    // determinism
    assert_eq!(v, run_cqt(repo.path(), &args));
}
