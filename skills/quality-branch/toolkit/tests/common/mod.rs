//! Test harness: build a throwaway git repo, drive the real compiled `cqt`
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

pub fn run_cqt(repo: &Path, args: &[&str]) -> serde_json::Value {
    let mut full = vec!["--repo".to_string(), repo.display().to_string()];
    full.extend(args.iter().map(|s| s.to_string()));
    let out = Command::new(env!("CARGO_BIN_EXE_cqt"))
        .args(&full)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "cqt {args:?} failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    serde_json::from_slice(&out.stdout).expect("stdout must be a single JSON document")
}
