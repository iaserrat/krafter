mod common;
use common::{candidate, run_ctk, Repo};

/// Removing a symbol that was `#[deprecated]` at base is still breaking, but is
/// reported under the distinct, lower-priority `REMOVED_DEPRECATED` reason and
/// sorts below a fresh removal.
#[test]
fn deprecated_removal_is_distinct_and_lower_priority() {
    let repo = Repo::new();
    repo.write(
        "l.rs",
        "#[deprecated(note = \"use v2\")]\npub fn old_api() {}\npub fn fresh(a: i32) -> i32 { a }\n",
    );
    repo.commit_all("base");

    repo.checkout_new("feature");
    repo.write("l.rs", "pub fn other() {}\n");
    repo.commit_all("remove both");

    let v = run_ctk(repo.path(), &["--base", "main", "assess"]);
    assert_eq!(candidate(&v, "old_api").unwrap()["reason"], "REMOVED_DEPRECATED");
    assert_eq!(candidate(&v, "fresh").unwrap()["reason"], "REMOVED");

    // fresh (REMOVED) is ranked above old_api (REMOVED_DEPRECATED).
    let order: Vec<&str> = v["candidates"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c["symbol"].as_str().unwrap())
        .collect();
    let fresh = order.iter().position(|s| *s == "fresh").unwrap();
    let old = order.iter().position(|s| *s == "old_api").unwrap();
    assert!(fresh < old, "fresh removal ranks above deprecated removal");
}
