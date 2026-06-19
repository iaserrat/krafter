mod common;
use common::{run_ctk_code, Repo};

fn breaking_repo() -> Repo {
    let repo = Repo::new();
    repo.write("l.rs", "pub fn keep() {}\npub fn gone(a: i32) -> i32 { a }\n");
    repo.commit_all("base");
    repo.checkout_new("feature");
    repo.write("l.rs", "pub fn keep() {}\n");
    repo.commit_all("remove gone");
    repo
}

/// Report-only by default: a breaking change still exits 0 (the agent reads JSON).
#[test]
fn gate_is_report_only_without_fail_on() {
    let r = run_ctk_code(breaking_repo().path(), &["--base", "main", "assess"]);
    assert_eq!(r.json["semver_impact"], "major");
    assert_eq!(r.code, 0, "no --fail-on means no gate");
}

/// `--fail-on major` exits non-zero on a breaking change.
#[test]
fn gate_fails_on_major() {
    let r = run_ctk_code(breaking_repo().path(), &["--base", "main", "assess", "--fail-on", "major"]);
    assert_eq!(r.code, 2);
    assert_eq!(r.json["gate"]["tripped"], true);
}

/// A baselined break is excluded from the gate decision (exit 0) but stays in the
/// reported JSON with `suppressed: true` and the headline impact unchanged.
#[test]
fn gate_baseline_suppresses_known_break() {
    let repo = breaking_repo();
    repo.write("accepted.txt", "# accepted breaks\nREMOVED l.rs gone\n");
    let baseline = repo.path().join("accepted.txt").display().to_string();
    let r = run_ctk_code(
        repo.path(),
        &["--base", "main", "assess", "--fail-on", "major", "--baseline", &baseline],
    );
    assert_eq!(r.code, 0, "baselined break must not trip the gate");
    assert_eq!(r.json["gate"]["tripped"], false);
    assert_eq!(r.json["gate"]["suppressed"], 1);
    assert_eq!(r.json["semver_impact"], "major", "report still shows the truth");
}
