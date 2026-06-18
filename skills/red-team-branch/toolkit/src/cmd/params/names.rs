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
    "id",
    "user",
    "user_id",
    "uid",
    "account",
    "page",
    "per_page",
    "limit",
    "offset",
    "q",
    "query",
    "search",
    "sort",
    "order",
    "fields",
    "include",
    "expand",
    "filter",
    "format",
    "callback",
    "jsonp",
    "redirect",
    "redirect_uri",
    "next",
    "url",
    "uri",
    "file",
    "path",
    "template",
    "lang",
    "locale",
    "debug",
    "test",
    "admin",
    "is_admin",
    "role",
    "token",
    "api_key",
    "access_token",
    "key",
    "secret",
    "preview",
    "draft",
    "force",
    "all",
    "raw",
];
