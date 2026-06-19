//! Test harness: build a throwaway git repo, drive the real compiled `ctk`
//! binary against it, and parse its JSON stdout (the agent's contract).
// Each test file is its own crate, so some helpers are unused per-crate.
#![allow(dead_code)]

use std::path::Path;
use std::process::Command;
use tempfile::TempDir;

pub struct Repo {
    dir: TempDir,
}

impl Repo {
    pub fn new() -> Repo {
        let dir = tempfile::tempdir().unwrap();
        git(dir.path(), &["init", "-q", "-b", "main"]);
        git(dir.path(), &["config", "user.email", "t@example.com"]);
        git(dir.path(), &["config", "user.name", "Test"]);
        Repo { dir }
    }

    pub fn path(&self) -> &Path {
        self.dir.path()
    }

    pub fn write(&self, rel: &str, content: &str) {
        let p = self.path().join(rel);
        if let Some(parent) = p.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(p, content).unwrap();
    }

    pub fn remove(&self, rel: &str) {
        std::fs::remove_file(self.path().join(rel)).unwrap();
    }

    pub fn commit_all(&self, msg: &str) {
        git(self.path(), &["add", "-A"]);
        git(self.path(), &["commit", "-q", "-m", msg]);
    }

    pub fn checkout_new(&self, branch: &str) {
        git(self.path(), &["checkout", "-q", "-b", branch]);
    }
}

pub fn git(repo: &Path, args: &[&str]) {
    let status = Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(args)
        .env("GIT_AUTHOR_DATE", "2020-01-01T00:00:00")
        .env("GIT_COMMITTER_DATE", "2020-01-01T00:00:00")
        .status()
        .unwrap();
    assert!(status.success(), "git {args:?} failed");
}

pub fn run_ctk(repo: &Path, args: &[&str]) -> serde_json::Value {
    let mut full = vec!["--repo".to_string(), repo.display().to_string()];
    full.extend(args.iter().map(|s| s.to_string()));
    let out = Command::new(env!("CARGO_BIN_EXE_ctk"))
        .args(&full)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "ctk {args:?} failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    serde_json::from_slice(&out.stdout).expect("stdout must be a single JSON document")
}

/// JSON + exit code for gate tests (does not assert success).
pub struct Run {
    pub json: serde_json::Value,
    pub code: i32,
}

pub fn run_ctk_code(repo: &Path, args: &[&str]) -> Run {
    let mut full = vec!["--repo".to_string(), repo.display().to_string()];
    full.extend(args.iter().map(|s| s.to_string()));
    let out = Command::new(env!("CARGO_BIN_EXE_ctk"))
        .args(&full)
        .output()
        .unwrap();
    Run {
        json: serde_json::from_slice(&out.stdout).expect("stdout must be JSON"),
        code: out.status.code().unwrap_or(-1),
    }
}

/// Candidate matching `symbol` substring in an `assess` result.
pub fn candidate<'a>(v: &'a serde_json::Value, symbol: &str) -> Option<&'a serde_json::Value> {
    let arr = v["candidates"].as_array()?;
    arr.iter()
        .find(|c| c["symbol"].as_str().unwrap_or("").contains(symbol))
}
