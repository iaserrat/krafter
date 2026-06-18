mod common;
use common::{run_cqt, Repo};

// Structurally identical, identifiers + literals renamed: a Type-2 clone.
// Dense enough to clear the 50-token / 5-line floors.
const FILE_A: &str = "fn total_price(items: &[i32], rate: i32, base: i32) -> i32 {\n\
    let mut sum = base;\n\
    for value in items {\n\
        if *value > 0 { sum = sum + value * rate + base; } else { sum = sum - value - rate; }\n\
    }\n\
    return sum;\n\
}\n";

const FILE_B: &str = "fn aggregate_cost(rows: &[i32], factor: i32, start: i32) -> i32 {\n\
    let mut acc = start;\n\
    for entry in rows {\n\
        if *entry > 0 { acc = acc + entry * factor + start; } else { acc = acc - entry - factor; }\n\
    }\n\
    return acc;\n\
}\n";

fn paths(group: &serde_json::Value) -> Vec<String> {
    let mut p: Vec<String> = group["members"]
        .as_array()
        .unwrap()
        .iter()
        .map(|m| m["path"].as_str().unwrap().to_string())
        .collect();
    p.sort();
    p
}

#[test]
fn dup_detects_type2_clone_across_files() {
    let repo = Repo::new();
    repo.write("a.rs", FILE_A);
    repo.write("b.rs", FILE_B);
    repo.commit_all("two near-identical files");

    let v = run_cqt(repo.path(), &["dup", "--scope", "repo"]);
    let groups = v["clone_groups"].as_array().unwrap();
    assert_eq!(groups.len(), 1);
    assert_eq!(paths(&groups[0]), vec!["a.rs", "b.rs"]);
    assert!(groups[0]["token_length"].as_u64().unwrap() >= 50);
}

#[test]
fn dup_groups_all_files_in_a_three_way_clone() {
    let repo = Repo::new();
    repo.write("a.rs", FILE_A);
    repo.write("b.rs", FILE_B);
    let file_c = FILE_B
        .replace("aggregate_cost", "tally_up")
        .replace("rows", "list")
        .replace("factor", "scale")
        .replace("start", "seed")
        .replace("acc", "total")
        .replace("entry", "item");
    repo.write("c.rs", &file_c);
    repo.commit_all("three near-identical files");

    let v = run_cqt(repo.path(), &["dup", "--scope", "repo"]);
    let groups = v["clone_groups"].as_array().unwrap();
    assert_eq!(groups.len(), 1, "one region => one group");
    assert_eq!(paths(&groups[0]), vec!["a.rs", "b.rs", "c.rs"], "all three join");
}

#[test]
fn dup_type1_does_not_flag_renamed_clone() {
    let repo = Repo::new();
    repo.write("a.rs", FILE_A);
    repo.write("b.rs", FILE_B);
    repo.commit_all("base");
    let v = run_cqt(repo.path(), &["dup", "--scope", "repo", "--type1"]);
    assert_eq!(v["clone_groups"].as_array().unwrap().len(), 0);
}

#[test]
fn dup_branch_scope_only_reports_branch_clones() {
    let repo = Repo::new();
    repo.write("orig.rs", FILE_A);
    repo.commit_all("base");
    repo.checkout_new("feature");
    repo.write("copy.rs", FILE_B);
    repo.commit_all("introduce a clone");

    let v = run_cqt(repo.path(), &["--base", "main", "dup"]);
    assert_eq!(v["scope"], "branch");
    assert_eq!(v["clone_groups"].as_array().unwrap().len(), 1);
    assert_eq!(v, run_cqt(repo.path(), &["--base", "main", "dup"])); // deterministic
}
