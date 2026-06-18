pub fn resolve_url(base: Option<&str>, target: &str) -> anyhow::Result<String> {
    if target.starts_with("http://") || target.starts_with("https://") {
        return Ok(target.to_string());
    }
    let Some(base) = base else {
        anyhow::bail!("no base_url configured; pass a full URL or set http.base_url")
    };
    Ok(format!(
        "{}/{}",
        base.trim_end_matches('/'),
        target.trim_start_matches('/')
    ))
}
