pub const DEFAULT_PORTS: &[u16] = &[
    80, 443, 1080, 2375, 3000, 3001, 3306, 4000, 4200, 5000, 5173, 5432, 5601, 6379, 7000, 8000,
    8025, 8080, 8081, 8443, 8888, 9000, 9090, 9200, 11211, 15672, 27017,
];

pub fn parse_headers(raw: &[String]) -> Vec<(String, String)> {
    raw.iter()
        .filter_map(|h| h.split_once(':'))
        .map(|(k, v)| (k.trim().to_string(), v.trim().to_string()))
        .collect()
}

pub fn parse_ports(spec: &str) -> Vec<u16> {
    let mut ports = Vec::new();
    for tok in spec.split([',', ' ', '\t', '\n']).filter(|t| !t.is_empty()) {
        if let Some((a, b)) = tok.split_once('-') {
            if let (Ok(a), Ok(b)) = (a.trim().parse::<u16>(), b.trim().parse::<u16>()) {
                ports.extend(a.min(b)..=a.max(b));
            }
        } else if let Ok(p) = tok.trim().parse::<u16>() {
            ports.push(p);
        }
    }
    ports.sort_unstable();
    ports.dedup();
    ports
}

pub fn default_ports() -> Vec<u16> {
    DEFAULT_PORTS.to_vec()
}
