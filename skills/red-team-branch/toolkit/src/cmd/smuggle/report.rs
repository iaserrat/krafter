use super::args::Args;
use super::probe::Technique;
use super::raw::RawOutcome;

pub struct Finding {
    pub technique: String,
    pub smuggle_status: u16,
    pub followup_status: u16,
    pub verdict: &'static str,
    pub evidence: String,
}

pub fn emit(args: &Args, target: &str, findings: &[Finding]) {
    let verdict = if findings.iter().any(|f| f.verdict == "EXPLOITABLE") {
        "SMUGGLING CONFIRMED"
    } else {
        "NO SMUGGLING DETECTED"
    };
    let results: Vec<serde_json::Value> = findings
        .iter()
        .map(|f| {
            serde_json::json!({
                "technique": f.technique,
                "smuggle_status": f.smuggle_status,
                "followup_status": f.followup_status,
                "verdict": f.verdict,
                "evidence": f.evidence,
            })
        })
        .collect();
    super::super::emit(&serde_json::json!({
        "probe": "smuggle",
        "target": target,
        "canary": args.canary_path,
        "verdict": verdict,
        "techniques_probed": findings.len(),
        "exploitable_count": findings.iter().filter(|f| f.verdict == "EXPLOITABLE").count(),
        "results": results,
    }));
}

pub fn classify(smuggle: &RawOutcome, followup: &RawOutcome, technique: &Technique, canary: &str) -> Finding {
    if smuggle.error.is_some() || followup.error.is_some() {
        return Finding {
            technique: technique.name.to_string(),
            smuggle_status: smuggle.status,
            followup_status: followup.status,
            verdict: "ERROR",
            evidence: format!("smuggle_err={:?} followup_err={:?}", smuggle.error, followup.error),
        };
    }
    // Each request travels on its own short-lived socket (Connection: close), so a
    // poisoned back-end queue cannot carry to the follow-up: the only sound desync
    // signal is the smuggled canary surfacing in the follow-up body. A bare
    // status flip is benign flakiness (overload, reset) and must NOT cry wolf.
    let followup_body = String::from_utf8_lossy(&followup.body_raw);
    let canary_reflected = !canary.is_empty() && followup_body.contains(canary);
    Finding {
        technique: technique.name.to_string(),
        smuggle_status: smuggle.status,
        followup_status: followup.status,
        verdict: if canary_reflected { "EXPLOITABLE" } else { "SAFE" },
        evidence: evidence(canary_reflected, canary),
    }
}

fn evidence(canary_reflected: bool, canary: &str) -> String {
    if canary_reflected {
        format!("smuggled canary '{canary}' reflected in follow-up")
    } else {
        "CL/TE constructed; follow-up clean — no desync observed".to_string()
    }
}
