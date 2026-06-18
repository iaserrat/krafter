use super::{Args, CANARY};
use crate::http;
use serde_json::Value;

pub fn spec(
    args: &Args,
    base: &str,
    headers: &[(String, String)],
    name: &str,
) -> http::RequestSpec {
    let injection = inject(args, base, name);
    http::RequestSpec::new(&args.method, injection.url)
        .with_text_headers(headers)
        .with_body(injection.body)
}

struct Injection {
    url: String,
    body: Option<String>,
}

fn inject(args: &Args, base: &str, name: &str) -> Injection {
    match args.location.as_str() {
        "query" => query(base, name),
        "json" => json_body(args, base, name),
        _ => form_body(args, base, name),
    }
}

fn query(base: &str, name: &str) -> Injection {
    let sep = if base.contains('?') { '&' } else { '?' };
    Injection {
        url: format!("{base}{sep}{name}={CANARY}"),
        body: None,
    }
}

fn json_body(args: &Args, base: &str, name: &str) -> Injection {
    let mut obj = args
        .body
        .as_deref()
        .and_then(|b| serde_json::from_str::<Value>(b).ok())
        .and_then(|v| v.as_object().cloned())
        .unwrap_or_default();
    obj.insert(name.to_string(), Value::String(CANARY.into()));
    Injection {
        url: base.to_string(),
        body: Some(Value::Object(obj).to_string()),
    }
}

fn form_body(args: &Args, base: &str, name: &str) -> Injection {
    let body = match &args.body {
        Some(body) if !body.is_empty() => format!("{body}&{name}={CANARY}"),
        _ => format!("{name}={CANARY}"),
    };
    Injection {
        url: base.to_string(),
        body: Some(body),
    }
}
