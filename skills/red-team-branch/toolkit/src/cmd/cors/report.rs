use super::probe::CorsResult;

pub fn emit(results: &[CorsResult], issues_per_origin: &[Vec<String>]) {
    let total_issues: usize = issues_per_origin.iter().map(|i| i.len()).sum();
    let verdict = if total_issues > 0 { "CORS MISCONFIGURATION" } else { "CORS SAFE" };
    let result_json: Vec<serde_json::Value> = results
        .iter()
        .zip(issues_per_origin.iter())
        .map(|(r, issues)| {
            serde_json::json!({
                "origin": r.origin,
                "acao": r.acao,
                "acac": r.acac,
                "status": r.status,
                "issues": issues,
            })
        })
        .collect();
    super::super::emit(&serde_json::json!({
        "probe": "cors",
        "verdict": verdict,
        "origins_probed": results.len(),
        "total_issues": total_issues,
        "results": result_json,
    }));
}

const CREDS_TRUE: &str = "true";
const WILDCARD: &str = "*";
const NULL: &str = "null";

pub fn classify(r: &CorsResult) -> Vec<String> {
    let mut issues = Vec::new();
    if let Some(ref acao) = r.acao {
        if acao.as_str() == r.origin {
            issues.push("origin-reflected".to_string());
        }
        if acao == WILDCARD {
            issues.push("wildcard-allow-origin".to_string());
        }
        if acao == NULL && r.origin == *NULL {
            issues.push("null-origin-accepted".to_string());
        }
    }
    if creds_allowed(r) {
        // ACAC:true is inert unless ACAO is reflected/null/wildcard; a fixed
        // own-origin allowlist that never reflects the attacker is SAFE.
        if r.acao.as_deref() == Some(WILDCARD) {
            issues.push("wildcard-with-credentials".to_string());
        }
        issues.push("credentials-allowed".to_string());
    }
    issues
}

// Credentials matter only when the responding ACAO grants the *request* origin.
fn creds_allowed(r: &CorsResult) -> bool {
    let acac_true = r.acac.as_deref().is_some_and(|v| v.eq_ignore_ascii_case(CREDS_TRUE));
    acac_true && acao_grants_origin(r)
}

fn acao_grants_origin(r: &CorsResult) -> bool {
    r.acao.as_deref().is_some_and(|acao| acao == WILDCARD || acao == r.origin)
}

// A preflight is accepted only when the response actually grants the probe
// origin AND lists the requested method in ACAM; status alone proves nothing.
pub fn preflight_accepted(r: &CorsResult, method: &str) -> bool {
    let method_allowed = r.acam.as_deref().is_some_and(|acam| {
        acam.split(',').any(|m| m.trim().eq_ignore_ascii_case(method))
    });
    acao_grants_origin(r) && method_allowed
}
