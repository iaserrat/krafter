//! Pair functions across two analyses (base vs HEAD) by qualified name.
//! Collision-safe: two functions sharing a name (overloads, two impl blocks,
//! macro expansions) are disambiguated by a stable same-name ordinal, so one
//! never silently overwrites the other. Shared by `delta` and `assess`.

use crate::engine::model::FunctionMetrics;
use std::collections::{BTreeMap, HashMap};

const ORD_SEP: char = '#';

#[derive(Clone, Copy, PartialEq)]
pub enum Change {
    Added,
    Removed,
    Changed,
}

/// A matched function across the two sides. Exactly one of before/after is None
/// for Added/Removed; both are Some for Changed.
pub struct FnPair {
    pub path: String,
    pub name: String,
    pub change: Change,
    pub before: Option<FunctionMetrics>,
    pub after: Option<FunctionMetrics>,
}

/// Pair `before` and `after` function lists for one file. Deterministic: keys
/// are sorted and the same-name ordinal is assigned by start_line.
pub fn pair_functions(path: &str, before: &[FunctionMetrics], after: &[FunctionMetrics]) -> Vec<FnPair> {
    let b = index(before);
    let a = index(after);
    let mut keys: Vec<&String> = b.keys().chain(a.keys()).collect();
    keys.sort();
    keys.dedup();
    keys.into_iter()
        .map(|k| {
            let before = b.get(k).map(|m| (*m).clone());
            let after = a.get(k).map(|m| (*m).clone());
            let change = match (before.is_some(), after.is_some()) {
                (false, true) => Change::Added,
                (true, false) => Change::Removed,
                _ => Change::Changed,
            };
            FnPair {
                path: path.to_string(),
                name: display_name(k),
                change,
                before,
                after,
            }
        })
        .collect()
}

fn index(fns: &[FunctionMetrics]) -> BTreeMap<String, &FunctionMetrics> {
    let mut sorted: Vec<&FunctionMetrics> = fns.iter().collect();
    sorted.sort_by(|x, y| {
        x.name
            .cmp(&y.name)
            .then_with(|| x.start_line.cmp(&y.start_line))
    });
    let mut seen: HashMap<&str, usize> = HashMap::new();
    let mut map = BTreeMap::new();
    for m in sorted {
        let ord = seen.entry(m.name.as_str()).or_insert(0);
        map.insert(format!("{}{ORD_SEP}{ord}", m.name), m);
        *ord += 1;
    }
    map
}

/// Strip the `#<ord>` disambiguator back off for display.
fn display_name(key: &str) -> String {
    match key.rsplit_once(ORD_SEP) {
        Some((name, _)) => name.to_string(),
        None => key.to_string(),
    }
}
