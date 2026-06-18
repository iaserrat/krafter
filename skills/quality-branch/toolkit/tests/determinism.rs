mod common;
use common::{run_cqt, Repo};

// Exercise the parallel repo scan (>= PARALLEL_MIN_FILES) and assert two runs
// produce byte-identical JSON — the determinism contract under threading.
#[test]
fn calibrate_is_byte_identical_across_runs() {
    let repo = Repo::new();
    for i in 0..24 {
        repo.write(
            &format!("f{i}.rs"),
            &format!("fn g{i}(a: i32) -> i32 {{ if a > {i} {{ if a > 1 {{ a }} else {{ 0 }} }} else {{ 0 }} }}"),
        );
    }
    repo.commit_all("many files");

    let a = run_cqt(repo.path(), &["calibrate"]);
    let b = run_cqt(repo.path(), &["calibrate"]);
    assert_eq!(a, b, "parallel scan must be deterministic");
    assert!(a["functions"].as_u64().unwrap() >= 24);
}
