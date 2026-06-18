pub fn is_sensitive_path(path: &str) -> bool {
    let path = path.to_ascii_lowercase();
    HOT.iter().any(|needle| path.contains(needle))
}

const HOT: &[&str] = &[
    "admin", "debug", ".env", ".git", "backup", "actuator", "console", "internal", "private",
    "config", "phpinfo", "metrics", "graphql", "swagger", "openapi",
];
