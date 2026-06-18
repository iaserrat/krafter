use crate::{cmd, config::Ctx, http};
use serde_json::json;

const BATCH_MIN_RESULTS: usize = 2;

// A GraphQL server returns 200 even for refused operations; the real signal is
// whether the response carries executed `data`, not the HTTP status.
fn has_data(body: &[u8]) -> bool {
    serde_json::from_slice::<serde_json::Value>(body)
        .ok()
        .is_some_and(|v| value_has_data(&v))
}

// One operation executed iff it carries a non-null `data` field.
fn value_has_data(v: &serde_json::Value) -> bool {
    v.get("data").is_some_and(|d| !d.is_null())
}

pub async fn introspect(client: &reqwest::Client, ctx: &Ctx, url: &str, extra: &[(String, String)]) -> anyhow::Result<(bool, usize)> {
    let query = json!({"query": INTROSPECTION_QUERY});
    let spec = gql_request(ctx, url, "POST", &query.to_string(), extra)?;
    let out = http::send_once(client, &spec, http::NO_SNIPPET_LEN).await;
    let field_count = serde_json::from_slice::<serde_json::Value>(&out.body_raw)
        .ok()
        .and_then(|v| v.pointer("/data/__schema/types").and_then(|t| t.as_array()).map(|a| a.len()))
        .unwrap_or(0);
    Ok((field_count > 0, field_count))
}

pub async fn batch(client: &reqwest::Client, ctx: &Ctx, url: &str, extra: &[(String, String)]) -> anyhow::Result<bool> {
    let batch = json!([
        {"query": "{ __typename }"},
        {"query": "{ __schema { types { name } } }"}
    ]);
    let spec = gql_request(ctx, url, "POST", &batch.to_string(), extra)?;
    let out = http::send_once(client, &spec, http::NO_SNIPPET_LEN).await;
    // Batching is enabled only if the server executed each batched operation:
    // a >=2-element array where every element carries non-null `data`. An array
    // of pure errors means the batch was received but rejected, not enabled.
    let executed = serde_json::from_slice::<serde_json::Value>(&out.body_raw)
        .ok()
        .and_then(|v| v.as_array().cloned())
        .is_some_and(|a| a.len() >= BATCH_MIN_RESULTS && a.iter().all(value_has_data));
    Ok(executed)
}

pub async fn alias_bypass(client: &reqwest::Client, ctx: &Ctx, url: &str, count: usize, extra: &[(String, String)]) -> anyhow::Result<(bool, usize)> {
    let aliases: Vec<String> = (0..count)
        .map(|i| format!("a{i}: __typename"))
        .collect();
    let query = format!("{{ {} }}", aliases.join(" "));
    let spec = gql_request(ctx, url, "POST", &json!({"query": query}).to_string(), extra)?;
    let out = http::send_once(client, &spec, http::NO_SNIPPET_LEN).await;
    let accepted = has_data(&out.body_raw);
    Ok((accepted, if accepted { count } else { 0 }))
}

pub async fn get_query(client: &reqwest::Client, ctx: &Ctx, url: &str, extra: &[(String, String)]) -> anyhow::Result<bool> {
    let encoded = "{__typename}".replace('{', "%7B").replace('}', "%7D");
    let get_url = format!("{url}?query={encoded}");
    let mut spec = cmd::base_spec(ctx, "GET", &get_url, &[], None)?;
    for (k, v) in extra {
        spec.headers.push(http::RequestHeader::Text(k.clone(), v.clone()));
    }
    let out = http::send_once(client, &spec, http::NO_SNIPPET_LEN).await;
    Ok(has_data(&out.body_raw))
}

fn gql_request(ctx: &Ctx, url: &str, method: &str, body: &str, extra: &[(String, String)]) -> anyhow::Result<http::RequestSpec> {
    let header_strs: Vec<String> = extra.iter().map(|(k, v)| format!("{k}: {v}")).collect();
    let mut spec = cmd::base_spec(ctx, method, url, &header_strs, Some(body.to_string()))?;
    spec.headers.push(http::RequestHeader::Text("Content-Type".into(), "application/json".into()));
    Ok(spec)
}

pub(crate) const INTROSPECTION_QUERY: &str = r#"query { __schema { types { name fields { name } } } }"#;
