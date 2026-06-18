pub struct GqlFindings {
    pub introspection_enabled: bool,
    pub field_count: usize,
    pub batching_enabled: bool,
    pub alias_accepted: bool,
    pub alias_count: usize,
    pub get_query_enabled: bool,
    pub issues: Vec<String>,
}

pub fn emit(f: &GqlFindings) {
    let mut issues = f.issues.clone();
    if f.introspection_enabled { issues.push("introspection-enabled".into()); }
    if f.batching_enabled { issues.push("query-batching-enabled".into()); }
    if f.alias_accepted { issues.push("alias-rate-limit-bypass-possible".into()); }
    if f.get_query_enabled { issues.push("get-based-queries-enabled".into()); }
    let verdict = if issues.is_empty() { "GRAPHQL SAFE" } else { "GRAPHQL MISCONFIGURATION" };
    super::super::emit(&serde_json::json!({
        "probe": "gql",
        "verdict": verdict,
        "introspection_enabled": f.introspection_enabled,
        "introspection_field_count": f.field_count,
        "batching_enabled": f.batching_enabled,
        "alias_accepted": f.alias_accepted,
        "alias_count": f.alias_count,
        "get_query_enabled": f.get_query_enabled,
        "issues": issues,
    }));
}
