//! Diff-scoped duplication. Tokenizes every tracked file, finds cross-file
//! clones, and (in branch scope) keeps only groups a branch-changed file
//! participates in — i.e. duplication the branch introduced, including copies
//! of pre-existing repo code.

pub mod args;
pub mod detect;
pub mod extend;
pub mod fingerprint;
pub mod keywords;
pub mod lex;
pub mod model;
pub mod region;
pub mod token;

pub use args::Args;

use crate::cmd::{self, Ctx};
use crate::engine::scan;
use crate::git;
use detect::{detect, FileTokens};
use rust_code_analysis::get_language_for_file;
use serde_json::json;
use std::collections::HashSet;
use std::path::Path;
use token::{tokenize, CommentStyle};

const SCOPE_BRANCH: &str = "branch";

pub fn run(a: Args, ctx: &Ctx) -> anyhow::Result<()> {
    if a.k == 0 || a.min_tokens == 0 || a.min_lines == 0 {
        anyhow::bail!("--k, --min-tokens, and --min-lines must each be >= 1");
    }
    let listing = git::git(&ctx.repo, &["ls-files"])?;
    let type2 = !a.type1;
    let mut files = Vec::new();
    for rel in listing.lines() {
        let Some(lang) = get_language_for_file(Path::new(rel)) else {
            continue;
        };
        let Some(bytes) = scan::read_source(&ctx.repo.join(rel)) else {
            continue;
        };
        let name = format!("{lang:?}");
        let toks = tokenize(&bytes, comment_style(&name), type2);
        files.push(FileTokens {
            path: rel.to_string(),
            lang: name,
            lines: toks.iter().map(|t| t.line).collect(),
            kg: fingerprint::kgrams(&toks.iter().map(|t| t.hash).collect::<Vec<_>>(), a.k),
        });
    }
    let mut groups = detect(&files, a.k, a.min_tokens, a.min_lines);
    if a.scope == SCOPE_BRANCH {
        let branch: HashSet<String> = git::changed_paths(&ctx.repo, &ctx.base)
            .unwrap_or_default()
            .into_iter()
            .collect();
        groups.retain(|g| g.members.iter().any(|m| branch.contains(&m.path)));
    }
    cmd::emit(&json!({
        "command": "dup",
        "schema": "cqt.dup/v1",
        "scope": a.scope,
        "params": { "k": a.k, "min_tokens": a.min_tokens, "min_lines": a.min_lines, "type2": type2 },
        "clone_groups": groups,
    }));
    Ok(())
}

fn comment_style(lang: &str) -> CommentStyle {
    match lang {
        "Python" | "Ruby" => CommentStyle::Hash,
        _ => CommentStyle::Slash,
    }
}
