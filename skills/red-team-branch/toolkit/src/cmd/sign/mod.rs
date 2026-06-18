mod args;
mod crypto;
mod payload;

pub use args::Args;

use base64::Engine as _;
use serde_json::json;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn run(args: Args) -> anyhow::Result<()> {
    let payload = payload::read(&args)?;
    let signed = signed_input(&args, &payload);
    let raw = crypto::sign(
        args.algo.as_str(),
        args.secret.as_bytes(),
        signed.input.as_bytes(),
    )?;
    let signature = encode(args.encoding.as_str(), &raw)?;
    let header = header_value(args.format.as_str(), signed.timestamp, &signature);
    super::emit(&json!({
        "tool": "sign", "algo": args.algo, "encoding": args.encoding,
        "format": args.format, "timestamp": signed.timestamp, "signature": signature,
        "header_value": header, "payload_len": payload.len(),
        "hint": "send `header_value` in the receiver's expected signature header; also test wrong signatures",
    }));
    Ok(())
}

struct SignedInput {
    input: String,
    timestamp: Option<i64>,
}

fn signed_input(args: &Args, payload: &str) -> SignedInput {
    if args.format != "stripe" {
        return SignedInput {
            input: payload.to_string(),
            timestamp: None,
        };
    }
    let timestamp = args.timestamp.unwrap_or_else(now);
    SignedInput {
        input: format!("{timestamp}.{payload}"),
        timestamp: Some(timestamp),
    }
}

fn now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

fn encode(encoding: &str, raw: &[u8]) -> anyhow::Result<String> {
    Ok(match encoding {
        "hex" => hex::encode(raw),
        "base64" => base64::engine::general_purpose::STANDARD.encode(raw),
        other => anyhow::bail!("unsupported encoding '{other}'"),
    })
}

fn header_value(format: &str, timestamp: Option<i64>, signature: &str) -> String {
    match format {
        "stripe" => format!("t={},v1={}", timestamp.unwrap(), signature),
        _ => signature.to_string(),
    }
}
