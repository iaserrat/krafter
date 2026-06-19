mod common;
use common::{candidate, run_ctk, Repo};

const BASE: &str = "pub fn changed(a: i32) -> i32 { a }\n\
                    pub fn gone() {}\n\
                    pub fn shrink() {}\n";

/// The spine reason codes: a removed symbol, a changed signature, and a
/// narrowed visibility are each flagged, an added symbol is additive, and the
/// overall impact is `major`.
#[test]
fn assess_flags_every_breaking_reason() {
    let repo = Repo::new();
    repo.write("api.rs", BASE);
    repo.commit_all("base contract");

    repo.checkout_new("feature");
    repo.write(
        "api.rs",
        "pub fn changed(a: i32, b: i32) -> i32 { a + b }\n\
         fn shrink() {}\n\
         pub fn added() {}\n",
    );
    repo.commit_all("break the contract");

    let v = run_ctk(repo.path(), &["--base", "main", "assess"]);
    assert_eq!(v["command"], "assess");
    assert_eq!(v["semver_impact"], "major");

    assert_eq!(candidate(&v, "changed").unwrap()["reason"], "SIGNATURE_CHANGED");
    assert_eq!(candidate(&v, "gone").unwrap()["reason"], "REMOVED");
    assert_eq!(candidate(&v, "shrink").unwrap()["reason"], "VISIBILITY_REDUCED");
    let added = candidate(&v, "added").unwrap();
    assert_eq!(added["reason"], "ADDED");
    assert_eq!(added["breaking"], false);
    assert_eq!(added["semver"], "minor");

    // a signature change carries both sides as evidence.
    let changed = candidate(&v, "changed").unwrap();
    assert!(changed["before"].as_str().unwrap().contains("(a: i32)"));
    assert!(changed["after"].as_str().unwrap().contains("(a: i32, b: i32)"));
}

/// An additive-only branch is `minor`, never breaking.
#[test]
fn assess_additive_only_is_minor() {
    let repo = Repo::new();
    repo.write("api.rs", "pub fn one() {}\n");
    repo.commit_all("base");

    repo.checkout_new("feature");
    repo.write("api.rs", "pub fn one() {}\npub fn two(x: u8) -> u8 { x }\n");
    repo.commit_all("add surface");

    let v = run_ctk(repo.path(), &["--base", "main", "assess"]);
    assert_eq!(v["semver_impact"], "minor");
    assert!(v["candidates"].as_array().unwrap().iter().all(|c| c["breaking"] == false));
}

/// Deleting a file removes its public contract — a breaking change `assess`
/// must see (the base side has the symbols, the head side has none).
#[test]
fn assess_file_deletion_removes_contract() {
    let repo = Repo::new();
    repo.write("keep.rs", "pub fn keep() {}\n");
    repo.write("drop.rs", "pub fn dropped(a: i32) -> i32 { a }\n");
    repo.commit_all("base");

    repo.checkout_new("feature");
    repo.remove("drop.rs");
    repo.commit_all("delete drop.rs");

    let v = run_ctk(repo.path(), &["--base", "main", "assess"]);
    assert_eq!(v["semver_impact"], "major");
    let dropped = candidate(&v, "dropped").unwrap();
    assert_eq!(dropped["reason"], "REMOVED");
    assert_eq!(dropped["path"], "drop.rs");
}
