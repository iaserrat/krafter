use serde_json::Value;

pub struct ExtractedFields {
    pub keys: Vec<String>,
    pub sensitive: Vec<String>,
}

pub fn json_keys(body: &[u8]) -> ExtractedFields {
    let Ok(Value::Object(obj)) = serde_json::from_slice::<Value>(body) else {
        return ExtractedFields {
            keys: Vec::new(),
            sensitive: Vec::new(),
        };
    };
    let mut keys: Vec<String> = obj.keys().cloned().collect();
    keys.sort();
    keys.truncate(64);
    let sensitive = keys
        .iter()
        .filter(|key| is_sensitive(key))
        .cloned()
        .collect();
    ExtractedFields { keys, sensitive }
}

fn is_sensitive(key: &str) -> bool {
    let key = key.to_ascii_lowercase();
    key == "id" || key.ends_with("_id") || HINTS.iter().any(|hint| key.contains(hint))
}

const HINTS: &[&str] = &[
    "email",
    "phone",
    "ssn",
    "sin",
    "dob",
    "birth",
    "password",
    "passwd",
    "secret",
    "token",
    "api_key",
    "apikey",
    "authorization",
    "credit",
    "card",
    "cvv",
    "iban",
    "address",
    "health",
    "mrn",
    "patient",
    "salary",
    "hash",
    "private",
];
