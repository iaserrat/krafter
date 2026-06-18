mod common;
use common::{run_cqt, Repo};
use serde_json::Value;

fn reasons(c: &Value) -> Vec<String> {
    c["reasons"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r.as_str().unwrap().to_string())
        .collect()
}

#[test]
fn assess_ranks_regression_and_flags_biomarkers() {
    let repo = Repo::new();
    repo.write("pay.rs", "fn rate(a: i32) -> i32 { a }");
    repo.commit_all("base");

    repo.checkout_new("feature");
    repo.write(
        "pay.rs",
        "fn rate(a: i32) -> i32 { if a > 0 { if a > 1 { for _ in 0..a { if a > 3 { return a; } } } } 0 }\n\
         fn wide(a: i32, b: i32, c: i32, d: i32, e: i32, f: i32) -> i32 { a + b + c + d + e + f }",
    );
    repo.commit_all("complicate + wide signature");

    let v = run_cqt(repo.path(), &["--base", "main", "assess"]);
    assert_eq!(v["command"], "assess");
    assert_eq!(v["schema"], "cqt.assess/v1");
    let cands = v["candidates"].as_array().unwrap();

    // rate: a regression, evidence bundle is fully populated and rank is transparent.
    let rate = cands
        .iter()
        .find(|c| c["function"].as_str().unwrap().contains("rate"))
        .expect("rate is a candidate");
    assert_eq!(rate["direction"], "regression");
    assert!(reasons(rate).contains(&"COMPLEXITY_REGRESSION".to_string()));
    assert!(rate["delta_cognitive"].as_f64().unwrap() > 0.0);
    assert!(rate["cognitive_percentile"].as_f64().unwrap() > 0.0);
    assert!(rate["rank"]["score"].as_f64().unwrap() > 0.0);
    assert!(rate["rank"]["formula"].as_str().unwrap().contains("percentile"));

    // wide: new function with too many parameters -> biomarker reason.
    let wide = cands
        .iter()
        .find(|c| c["function"].as_str().unwrap().contains("wide"))
        .expect("wide is a candidate");
    assert!(reasons(&wide.clone()).contains(&"LARGE_ARG_LIST".to_string()));
    assert_eq!(wide["direction"], "new");
}

#[test]
fn assess_credits_cleanups_as_improved() {
    let repo = Repo::new();
    repo.write(
        "m.rs",
        "fn f(a: i32) -> i32 { if a > 0 { if a > 1 { if a > 2 { a } else { 0 } } else { 0 } } else { 0 } }",
    );
    repo.commit_all("base: tangled");

    repo.checkout_new("feature");
    repo.write("m.rs", "fn f(a: i32) -> i32 { a }");
    repo.commit_all("simplify");

    let v = run_cqt(repo.path(), &["--base", "main", "assess"]);
    let cands = v["candidates"].as_array().unwrap();
    let f = cands
        .iter()
        .find(|c| c["function"].as_str().unwrap().contains("f"))
        .expect("simplified f appears");
    assert_eq!(f["direction"], "improved");
    assert!(reasons(f).contains(&"IMPROVED".to_string()));
    assert!(f["delta_cognitive"].as_f64().unwrap() < 0.0);
}
