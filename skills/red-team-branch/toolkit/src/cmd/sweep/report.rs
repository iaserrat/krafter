use super::{args::Args, hit::Hit};
use crate::cmd;
use serde_json::json;
use std::collections::{BTreeMap, BTreeSet};

const MAX_RESULT_ROWS: usize = 200;

pub fn emit(args: &Args, url: String, tested: usize, hits: Vec<Hit>) {
    let summary = Summary::from_hits(&hits);
    let verdict = summary.verdict(args.compare.is_some(), hits.len());
    let results = hits
        .iter()
        .filter(|h| h.class != "denied")
        .take(MAX_RESULT_ROWS)
        .map(hit_json)
        .collect::<Vec<_>>();
    cmd::emit(&json!({
        "tool": "sweep", "url_template": url, "compare_profile": args.compare,
        "tested": tested, "by_class": summary.by_class,
        "distinct_private_bodies": summary.distinct_private.len(),
        "blocked_responses": summary.blocked,
        "exposed_fields": summary.key_union.into_iter().collect::<Vec<_>>(),
        "sensitive_fields": summary.sensitive_union.into_iter().collect::<Vec<_>>(),
        "verdict": verdict,
        "results": results,
    }));
}

struct Summary {
    by_class: BTreeMap<&'static str, usize>,
    distinct_private: BTreeSet<String>,
    key_union: BTreeSet<String>,
    sensitive_union: BTreeSet<String>,
    blocked: usize,
}

impl Summary {
    fn from_hits(hits: &[Hit]) -> Self {
        let mut s = Self {
            by_class: BTreeMap::new(),
            distinct_private: BTreeSet::new(),
            key_union: BTreeSet::new(),
            sensitive_union: BTreeSet::new(),
            blocked: 0,
        };
        for h in hits {
            *s.by_class.entry(h.class).or_default() += 1;
            if matches!(h.class, "accessible" | "cross_user_proven") {
                s.distinct_private.insert(h.sha8.clone());
            }
            s.blocked += h.blocked.is_some() as usize;
            s.key_union.extend(h.keys.iter().cloned());
            s.sensitive_union.extend(h.sensitive.iter().cloned());
        }
        s
    }

    fn verdict(&self, has_compare: bool, total: usize) -> String {
        let proven = *self.by_class.get("cross_user_proven").unwrap_or(&0);
        let accessible = *self.by_class.get("accessible").unwrap_or(&0);
        let public = *self.by_class.get("public").unwrap_or(&0);
        if self.blocked * 2 >= total && self.blocked > 0 {
            return "INCONCLUSIVE: majority blocked".into();
        }
        if proven > 0 {
            return format!(
                "PROVEN IDOR: actor A and compare actor both read {proven} private record(s)"
            );
        }
        if accessible >= 2 && self.distinct_private.len() >= 2 {
            return likely(has_compare);
        }
        if public >= 1 {
            return "PUBLIC EXPOSURE: unauthenticated control also reads these".into();
        }
        "no broad access from actor A".into()
    }
}

fn likely(has_compare: bool) -> String {
    if has_compare {
        "LIKELY IDOR: actor A reads multiple private records; seed compare-owned ids to prove"
            .into()
    } else {
        "LIKELY IDOR: actor A reads multiple private records; add --compare to prove".into()
    }
}

fn hit_json(h: &Hit) -> serde_json::Value {
    json!({"id": h.id, "class": h.class, "a_status": h.a_status, "anon_status": h.anon_status, "b_status": h.b_status, "body_len": h.body_len, "body_sha8": h.sha8, "fields": h.keys, "sensitive_fields": h.sensitive, "blocked": h.blocked, "snippet": h.snippet})
}
