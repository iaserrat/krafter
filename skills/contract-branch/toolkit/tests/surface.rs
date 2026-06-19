mod common;
use common::{run_ctk, Repo};

/// surface extracts public symbols with a normalized signature and excludes
/// non-public ones — the contract `assess` diffs.
#[test]
fn surface_lists_public_excludes_private() {
    let repo = Repo::new();
    repo.write(
        "api.rs",
        "pub fn alpha(a: i32, b: u8) -> i32 { a + b as i32 }\nfn helper() {}\npub(crate) fn internal() {}",
    );
    let abs = repo.path().join("api.rs").display().to_string();
    let v = run_ctk(repo.path(), &["surface", "--paths", &abs]);

    assert_eq!(v["command"], "surface");
    let syms = v["files"][0]["symbols"].as_array().unwrap();
    let pubs: Vec<&str> = syms
        .iter()
        .filter(|s| s["visibility"] == "public")
        .map(|s| s["name"].as_str().unwrap())
        .collect();
    assert!(pubs.iter().any(|n| n.contains("alpha")), "pub fn is public");
    assert!(!pubs.iter().any(|n| n.contains("helper")), "private excluded");
    assert!(
        !pubs.iter().any(|n| n.contains("internal")),
        "pub(crate) is not public contract"
    );
    let alpha = syms.iter().find(|s| s["name"].as_str().unwrap().contains("alpha")).unwrap();
    assert_eq!(alpha["signature"], "pub fn alpha(a: i32, b: u8) -> i32");
}

/// A language ctk has no contract rule for is reported unmeasured, never as an
/// empty contract (the honesty flag).
#[test]
fn surface_flags_unsupported_language() {
    let repo = Repo::new();
    repo.write("main.cpp", "int exported(int a) { return a; }");
    let abs = repo.path().join("main.cpp").display().to_string();
    let v = run_ctk(repo.path(), &["surface", "--paths", &abs]);

    assert_eq!(v["files"][0]["vis_supported"], false);
    assert!(v["unmeasured"].as_u64().unwrap() >= 1);
}
