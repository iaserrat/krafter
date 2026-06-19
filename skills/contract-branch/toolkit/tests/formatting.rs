mod common;
use common::{run_ctk, Repo};

/// Reformatting, reordering, and rewriting bodies must NOT read as a contract
/// change — the formatter-proof guarantee. Only real shape changes count.
#[test]
fn assess_ignores_formatting_and_body_churn() {
    let repo = Repo::new();
    repo.write(
        "api.rs",
        "/// old\npub fn reformat(a:i32,b:i32)->i32{a+b}\npub fn body(x: u8) -> u8 { x }\n",
    );
    repo.commit_all("base");

    repo.checkout_new("feature");
    repo.write(
        "api.rs",
        "pub fn body(x: u8) -> u8 {\n    let y = x;\n    y\n}\n\
         /// NEW doc\npub fn reformat(a: i32, b: i32) -> i32 {\n    a + b\n}\n",
    );
    repo.commit_all("rustfmt + reorder + body rewrite + doc change");

    let v = run_ctk(repo.path(), &["--base", "main", "assess"]);
    assert_eq!(v["semver_impact"], "none", "no contract change occurred");
    assert!(v["candidates"].as_array().unwrap().is_empty());
}

/// Reordering struct fields and enum variants, reformatting onto multiple lines,
/// and changing docs/attributes are not contract changes — members match by name.
#[test]
fn type_member_reorder_and_reformat_is_silent() {
    let repo = Repo::new();
    repo.write(
        "lib.rs",
        "/// old\npub struct C { pub host: String, pub map: HashMap<K, V> }\npub enum M { Fast, Slow }\n",
    );
    repo.commit_all("base");

    repo.checkout_new("feature");
    repo.write(
        "lib.rs",
        "/// NEW doc\n#[derive(Debug)]\npub struct C {\n    pub map: HashMap<K, V>,\n    pub host: String,\n}\npub enum M {\n    Slow,\n    Fast,\n}\n",
    );
    repo.commit_all("reorder + reformat + doc/attr churn");

    let v = run_ctk(repo.path(), &["--base", "main", "assess"]);
    assert_eq!(v["semver_impact"], "none");
    assert!(v["candidates"].as_array().unwrap().is_empty());
}
