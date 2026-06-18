use super::Args;

pub fn load(args: &Args) -> anyhow::Result<Vec<String>> {
    if let Some(path) = &args.wordlist {
        let text = std::fs::read_to_string(path)?;
        return Ok(text
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .map(str::to_string)
            .collect());
    }
    Ok(BUILTIN.iter().map(|s| s.to_string()).collect())
}

const BUILTIN: &[&str] = &[
    "admin",
    "administrator",
    "login",
    "api",
    "api/v1",
    "actuator",
    "actuator/health",
    "debug",
    "status",
    "metrics",
    "swagger.json",
    "swagger-ui",
    "openapi.json",
    ".env",
    ".git/config",
    ".git/HEAD",
    "config.json",
    "backup.zip",
    "backup.sql",
    "robots.txt",
    ".well-known/security.txt",
    "phpinfo.php",
    "server-status",
    "wp-admin",
    "console",
    "graphql",
    "internal",
    "private",
    "test",
    "dev",
];
