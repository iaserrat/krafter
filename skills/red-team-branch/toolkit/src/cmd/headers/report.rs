use super::check::HeaderReport;

pub fn emit(report: &HeaderReport) {
    if let Some(error) = &report.error {
        super::super::emit(&serde_json::json!({
            "probe": "headers",
            "verdict": "HEADERS ERROR",
            "error": error,
        }));
        return;
    }
    let verdict = if report.issues.is_empty() { "HEADERS SAFE" } else { "HEADER ISSUES FOUND" };
    let cookies: Vec<serde_json::Value> = report.set_cookie.iter().map(|c| {
        serde_json::json!({
            "name": c.name,
            "secure": c.secure,
            "http_only": c.http_only,
            "same_site": c.same_site,
        })
    }).collect();
    super::super::emit(&serde_json::json!({
        "probe": "headers",
        "verdict": verdict,
        "headers": report.headers,
        "cookie_count": report.set_cookie.len(),
        "cookies": cookies,
        "issue_count": report.issues.len(),
        "issues": report.issues,
    }));
}
