mod common;

#[test]
fn fuzz_state_resume_folds_prior_corpus() {
    let port = common::start_server();
    let cfg = common::write_config(port);
    let state = std::env::temp_dir().join(format!("rtk_state_{port}.json"));
    let _ = std::fs::remove_file(&state);
    let st = state.to_str().unwrap();
    let v1 = common::run_rtk(
        &cfg,
        &[
            "fuzz",
            "--mutate",
            "--url",
            "/records/{FUZZ}",
            "--seed",
            "1",
            "--max-exec",
            "150",
            "--state",
            st,
        ],
    );
    assert!(state.exists(), "state file should be written");
    let c1 = v1["stats"]["corpus_size"].as_u64().unwrap();
    let v2 = common::run_rtk(
        &cfg,
        &[
            "fuzz",
            "--mutate",
            "--url",
            "/records/{FUZZ}",
            "--seed",
            "2",
            "--max-exec",
            "150",
            "--state",
            st,
        ],
    );
    let c2 = v2["stats"]["corpus_size"].as_u64().unwrap();
    assert!(
        c2 >= c1,
        "resume should fold prior corpus: run1={c1} run2={c2}"
    );
    let _ = std::fs::remove_file(&state);
}
