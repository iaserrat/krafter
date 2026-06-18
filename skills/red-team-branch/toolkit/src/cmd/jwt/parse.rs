use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;

pub struct Jwt {
    pub payload: Vec<u8>,
    pub header_json: serde_json::Value,
}

pub fn parse(raw: &str) -> anyhow::Result<Jwt> {
    let parts: Vec<&str> = raw.splitn(3, '.').collect();
    if parts.len() < 2 {
        anyhow::bail!("not a JWT (expected header.payload[.signature])");
    }
    let header = decode_part(parts[0])?;
    let payload = decode_part(parts[1])?;
    let header_json: serde_json::Value =
        serde_json::from_slice(&header).map_err(|e| anyhow::anyhow!("malformed header json: {e}"))?;
    Ok(Jwt { payload, header_json })
}

fn decode_part(b64: &str) -> anyhow::Result<Vec<u8>> {
    URL_SAFE_NO_PAD.decode(b64).map_err(|e| anyhow::anyhow!("base64 decode: {e}"))
}

pub fn encode_part(bytes: &[u8]) -> String {
    URL_SAFE_NO_PAD.encode(bytes)
}
