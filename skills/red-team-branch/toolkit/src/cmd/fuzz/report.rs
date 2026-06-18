use super::{encoding, injector::Injector, session::Session};
use crate::{cmd, cmd::cov, http};
use serde_json::json;

const MAX_ANOMALIES: usize = 50;

pub fn emit(session: &Session) {
    let out = anomalies(session)
        .into_iter()
        .take(MAX_ANOMALIES)
        .map(|r| finding_json(r, &session.inj))
        .collect::<Vec<_>>();
    cmd::emit(&json!({
        "tool": "fuzz",
        "mode": "mutate",
        "seed": session.seed_val,
        "url_template": session.args.url,
        "channel": session.inj.channel_name(),
        "baseline": baseline(&session.base, session.lat_lower),
        "stats": stats(session),
        "anomalies": out,
    }));
}

fn anomalies(session: &Session) -> Vec<&cov::Repro> {
    let mut out = session.dedup.values().collect::<Vec<_>>();
    out.sort_by(|x, y| {
        cov::severity_rank(y.oracle_mask, y.family)
            .cmp(&cov::severity_rank(x.oracle_mask, x.family))
            .then(x.minimized.len().cmp(&y.minimized.len()))
    });
    out
}

fn finding_json(r: &cov::Repro, inj: &Injector) -> serde_json::Value {
    json!({
        "oracle_mask": r.oracle_mask,
        "reasons": cov::reasons(r.oracle_mask, r.family),
        "family": cov::family_name(r.family),
        "status": r.status,
        "body_len": r.body_len,
        "latency_ms": r.latency_ms,
        "payload_b64": encoding::b64(&r.payload),
        "payload_hex": hex::encode(&r.payload),
        "minimized_b64": encoding::b64(&r.minimized),
        "minimized_hex": hex::encode(&r.minimized),
        "minimized_len": r.minimized.len(),
        "found_at_exec": r.iter,
        "repro_curl": inj.repro(&r.minimized),
        "snippet": r.snippet,
    })
}

fn baseline(base: &http::Outcome, lat_lower: f64) -> serde_json::Value {
    json!({"status": base.status, "body_len": base.body_len, "lat_lower_ms": lat_lower})
}

fn stats(s: &Session) -> serde_json::Value {
    json!({"execs": s.exec, "unique_responses": s.seen.len(), "corpus_size": s.corpus.len(), "dict_size": s.dict.len(), "anomaly_buckets": s.dedup.len(), "rounds": s.rounds, "plateau_hit": s.plateau_hit})
}
