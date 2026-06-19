mod common;
use common::{run_ctk, Repo};

/// Same branch + same base -> byte-identical JSON. The contract is a function of
/// repo state only: no wall-clock, no ordering that depends on scheduling.
#[test]
fn assess_is_byte_identical_across_runs() {
    let repo = Repo::new();
    repo.write("a.rs", "pub fn a(x: i32) -> i32 { x }\npub fn keep() {}\n");
    repo.write("b.rs", "pub fn b() {}\n");
    repo.commit_all("base");

    repo.checkout_new("feature");
    repo.write("a.rs", "pub fn a(x: i32, y: i32) -> i32 { x + y }\nfn keep() {}\n");
    repo.write("b.rs", "pub fn b() {}\npub fn c(z: u8) -> u8 { z }\n");
    repo.commit_all("churn the contract");

    let first = run_ctk(repo.path(), &["--base", "main", "assess"]);
    let second = run_ctk(repo.path(), &["--base", "main", "assess"]);
    assert_eq!(first, second, "assess must be deterministic");
    assert_eq!(first["semver_impact"], "major");
}
