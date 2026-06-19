mod common;
use common::{candidate, run_ctk, Repo};

/// Type-level contracts: a removed public field, a retyped field, a removed
/// variant, and a removed public const are each flagged — not just functions.
#[test]
fn rust_type_member_changes_are_flagged() {
    let repo = Repo::new();
    repo.write(
        "lib.rs",
        "pub struct Config { pub host: String, pub port: u16 }\n\
         pub enum Mode { Fast, Slow }\n\
         pub const LIMIT: usize = 10;\n",
    );
    repo.commit_all("base");

    repo.checkout_new("feature");
    repo.write(
        "lib.rs",
        "pub struct Config { pub host: String, pub port: u32 }\n\
         pub enum Mode { Fast }\n",
    );
    repo.commit_all("retype port, drop Slow, drop LIMIT");

    let v = run_ctk(repo.path(), &["--base", "main", "assess"]);
    assert_eq!(v["semver_impact"], "major");
    assert_eq!(candidate(&v, "Config::port").unwrap()["reason"], "SIGNATURE_CHANGED");
    assert_eq!(candidate(&v, "Mode::Slow").unwrap()["reason"], "REMOVED");
    assert_eq!(candidate(&v, "LIMIT").unwrap()["reason"], "REMOVED");
    // unchanged public field is not reported.
    assert!(candidate(&v, "Config::host").is_none());
}

/// Narrowing a public field below `pub` removes it from the contract (flagged).
#[test]
fn rust_field_made_private_is_a_break() {
    let repo = Repo::new();
    repo.write("lib.rs", "pub struct S { pub token: String }\n");
    repo.commit_all("base");

    repo.checkout_new("feature");
    repo.write("lib.rs", "pub struct S { token: String }\n");
    repo.commit_all("make token private");

    let v = run_ctk(repo.path(), &["--base", "main", "assess"]);
    assert_eq!(v["semver_impact"], "major");
    assert!(candidate(&v, "S::token").is_some());
}

/// Adding a public type member is additive (minor), never breaking.
#[test]
fn rust_new_variant_is_additive() {
    let repo = Repo::new();
    repo.write("lib.rs", "pub enum E { A }\n");
    repo.commit_all("base");

    repo.checkout_new("feature");
    repo.write("lib.rs", "pub enum E { A, B }\n");
    repo.commit_all("add variant B");

    let v = run_ctk(repo.path(), &["--base", "main", "assess"]);
    assert_eq!(v["semver_impact"], "minor");
    assert_eq!(candidate(&v, "E::B").unwrap()["reason"], "ADDED");
}
