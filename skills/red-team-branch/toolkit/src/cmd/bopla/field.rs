use serde_json::Value;

pub fn base_object(body: &str) -> anyhow::Result<serde_json::Map<String, Value>> {
    let value: Value =
        serde_json::from_str(body).map_err(|e| anyhow::anyhow!("--body is not valid JSON: {e}"))?;
    value
        .as_object()
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("--body must be a JSON object"))
}

pub fn parse(raw: &str) -> anyhow::Result<(String, Value)> {
    let (key, value) = raw
        .split_once('=')
        .ok_or_else(|| anyhow::anyhow!("--field must be key=value"))?;
    let value = serde_json::from_str(value).unwrap_or_else(|_| Value::String(value.to_string()));
    Ok((key.trim().to_string(), value))
}

pub fn persisted(body: &[u8], key: &str, value: &Value) -> bool {
    matches!(serde_json::from_slice::<Value>(body), Ok(Value::Object(obj)) if obj.get(key) == Some(value))
}
