use super::attack::AttackToken;

pub struct Finding {
    pub attack: String,
    pub token: String,
    pub header: String,
    pub target_status: u16,
    pub verdict: &'static str,
}

pub fn emit(findings: &[Finding], control_status: u16) {
    let exploited = findings.iter().any(|f| f.verdict == "VULNERABLE");
    let verdict = if exploited { "JWT EXPLOITABLE" } else { "JWT SAFE" };
    let results: Vec<serde_json::Value> = findings
        .iter()
        .map(|f| {
            serde_json::json!({
                "attack": f.attack,
                "header": f.header,
                "token": &f.token[..f.token.len().min(120)],
                "target_status": f.target_status,
                "verdict": f.verdict,
            })
        })
        .collect();
    super::super::emit(&serde_json::json!({
        "probe": "jwt",
        "verdict": verdict,
        "control_status": control_status,
        "signature_validated": control_status >= REJECT_THRESHOLD,
        "attacks_probed": findings.len(),
        "vulnerable_count": findings.iter().filter(|f| f.verdict == "VULNERABLE").count(),
        "results": results,
    }));
}

const REJECT_THRESHOLD: u16 = 400;

/// A forgery is exploitable only if the endpoint rejects the invalid-signature
/// control (>=400) yet accepts this token (<400) — proof of a real bypass.
pub fn classify(attack: &AttackToken, status: u16, control_status: u16) -> Finding {
    let is_vuln = status < REJECT_THRESHOLD && control_status >= REJECT_THRESHOLD;
    Finding {
        attack: attack.name.to_string(),
        token: attack.token.clone(),
        header: attack.header.clone(),
        target_status: status,
        verdict: if is_vuln { "VULNERABLE" } else { "BLOCKED" },
    }
}
